## Context

`app/src-tauri/src/commands.rs` に2つのクリップボードクリア経路がある(詳細は proposal.md - Why を参照)。

- `clear_clipboard_if_owned(app, guard)`: `guard.should_clear(current)` で判定。`should_clear` は **`ClipboardGuard` が内部に保持している `last_written`(直近の書き込み値)** と `current` を比較する。一致すればクリアし、`guard.clear()` で無条件に内部状態を破棄する。`lock()` から呼ばれる。
- `copy_field` 内インライン: `tokio::time::sleep(30s)` 後、**このコマンド呼び出し自身が書き込んだ値をクロージャでキャプチャした `expected_for_clear`** と現在のクリップボードを比較する。一致すれば `guard.clear_if_matches(expected_for_clear)` を呼ぶ(`expected` と一致する場合のみguardをクリア)。

比較対象が「guardの現在値」か「呼び出し元がキャプチャした値」かという違いがあり、これが両者を単純結合できない理由になっている(下記 Decisions 参照)。

## Goals / Non-Goals

**Goals:**
- クリップボードクリアの判定・実行ロジックを単一の関数に統一する。
- `openspec/specs/credential-copy-actions/spec.md` が既に規定している「30秒後、値が変化していなければクリアする/変化していればクリアしない」という観測可能な振る舞いを一切変えない。
- 30秒以内に複数回コピーした場合に、後発のコピー値が先発コピーのタイマーによって早期にクリアされる回帰を防ぐ(既存実装でも回避されているが、単純な関数統合をすると壊れうる)。

**Non-Goals:**
- `ClipboardGuard` の内部データ構造(`Arc<Mutex<Option<String>>>`)自体の変更・再設計は行わない。
- クリップボードクリアの遅延時間(30秒)や、lock時の即時クリアという既存の挙動仕様の変更は行わない。
- パフォーマンス最適化(Mutexロック回数の削減等)は本変更のスコープ外。既存と同等のロック回数で構わない。

## Decisions

### 1. `clear_clipboard_if_owned` に `expected: &str` を明示的に渡す設計にする

**選択した設計:**
```rust
pub fn clear_clipboard_if_owned(app: &tauri::AppHandle, guard: &ClipboardGuard, expected: &str) {
    let Ok(current) = app.clipboard().read_text() else { return; };
    if current == expected {
        let _ = app.clipboard().write_text(String::new());
        guard.clear_if_matches(expected);
    }
}
```

**なぜ `guard.should_clear()`(guard内部値との比較)ではなく `expected` 引数を使うか:**

`copy_field` の遅延クリアタスクは、非同期タスクとして spawn された時点で「このタスクが書き込んだ値」を確定させる必要がある。`guard` は `AppState` 越しに全体で共有されているため、タスクの sleep(30秒)中に別の `copy_field` 呼び出しが発生すると `guard` の内部値は新しい値に上書きされる。

もし単純に `clear_clipboard_if_owned(app, guard)` を呼ぶ(＝内部で `guard.should_clear(current)` を使う)形に統合すると、以下の回帰が発生する:

```
t=0s   copy_field(password) → clipboard=V1, guard.set(V1), timer_A(30s)開始
t=10s  copy_field(username) → clipboard=V2, guard.set(V2), timer_B(30s)開始
t=30s  timer_A 発火
         現行仕様(維持すべき): expected=V1, current=V2 → 不一致 → 何もしない
         guard.should_clear()を使った場合: guard内部値はV2(Bが上書き済み) == current(V2) → true
           → 20秒しか経っていないV2を誤ってクリアしてしまう(回帰)
```

`expected` をタスク起動時にクロージャでキャプチャして渡すことで、このタスクが「自分が書き込んだ値」だけを対象に判定でき、上記の回帰を避けられる。

**`lock()` 側の対応:** `lock()` には「このコマンド呼び出しが書いた値」という概念がないため(ユーザー操作起点でguardの現在値を見るのが目的)、`ClipboardGuard::last_value() -> Option<String>` を新設し、guardの現在値を取得してから `expected` として渡す。

```rust
if let Some(expected) = guard.last_value() {
    clear_clipboard_if_owned(&app, &guard, &expected);
}
```

**検討した代替案:**
- **代替案A: `clear_clipboard_if_owned(app, guard)` のシグネチャを変えずそのまま `copy_field` から呼ぶ。**
  issue本文の記述に最も忠実だが、上記の競合状態を再現し「動作に変更がないこと」というacceptance criteriaに違反するため却下。
- **代替案B: `ClipboardGuard` 側に `clear_clipboard_if_owned` 相当のロジックを丸ごと持たせ、`app`(クリップボードI/O)への依存もguardに持たせる。**
  クリップボードI/O(`app.clipboard()`)への依存をguardに持ち込むとテスト容易性(`clipboard_guard.rs`の純粋ロジックとしてのユニットテスト)が損なわれるため却下。現状通り、guardは判定用の純粋な状態管理に留め、I/Oは`commands.rs`側に置く。

### 2. `guard.clear()`(無条件)ではなく `guard.clear_if_matches(expected)` に統一する

`lock()` 側も `clear_if_matches` 方式に変えることで、`lock()` がクリップボードを読んでから書き込むまでの間に別スレッドが `guard.set()` を呼んでも、意図しない値のguard状態を消してしまわない。これは既存の観測可能な通常動作(ユーザーから見た挙動)を変えるものではなく、既存にあった潜在的なTOCTOU的な穴を塞ぐ副次的な強化。

## Risks / Trade-offs

- [Risk] `expected` を引数化したことで呼び出し側(`lock()`, `copy_field`)双方に「呼び出し前にexpectedを用意する」責務が増える → [Mitigation] `lock()`側は `guard.last_value()` の一行、`copy_field`側は既存のクロージャキャプチャ(`expected_for_clear`)をそのまま流用するだけなので実質的な複雑度増加はない。
- [Risk] 回帰(連続コピー時の早期クリア)を防いだことをテストで担保しないと、将来また同じ問題が再発しうる → [Mitigation] tasks.md で、30秒以内に2回コピーした場合に後発の値が誤ってクリアされないことを検証する回帰テストの追加を明示する。

## Migration Plan

破壊的変更ではなく、`commands.rs` と `clipboard_guard.rs` の内部実装のみの変更(公開APIやUIへの影響なし)。段階的移行やロールバック手順は不要。通常のPRフロー(実装→レビュー→マージ)で完結する。

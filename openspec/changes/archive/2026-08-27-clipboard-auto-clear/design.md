## Context

`credential-actions-autolock` により `copy_field`(ユーザー名/パスワード/TOTPコピー)と `IdleTimer` によるアイドル自動ロック、`manual-lock` による手動ロックが実装済み。しかしコピーしたクリップボードの内容をクリアする仕組みは存在せず、平文の機密情報がクリップボードに残り続ける([GitHub Issue #80](https://github.com/kuchida1981/bitwarden-quickaccess/issues/80))。

コピー/ロックの実行は `app/src-tauri/src/commands.rs` の `copy_field` / `lock`、アイドル自動ロックは `app/src-tauri/src/main.rs` の `watch_idle_timeout` にそれぞれ実装されている。いずれもRustコア側で完結しており、WebView側のJSにクリップボードの平文値は渡していない(`credential-actions-autolock` design.md 決定1を踏襲)。本changeもこの制約を維持する。

## Goals / Non-Goals

**Goals:**
- `copy_field` で書き込んだ値を、遅延時間(30秒)経過後に自動的にクリアする。
- 手動ロック・アイドル自動ロック実行時にも、即座にクリップボードをクリアする。
- いずれのクリアも「クリップボードの中身が自分(アプリ)が書き込んだ値のままであること」を確認してから行い、ユーザーが別の値を既にコピーしている場合は上書き・誤消去しない。

**Non-Goals:**
- クリア遅延時間のユーザー設定UI・環境変数による上書き(既定値30秒固定。`IdleTimer::DEFAULT_IDLE_TIMEOUT` と同様の現行の慣習を踏襲)。
- クリア前のクリップボードの元の内容(コピー実行前に入っていた値)の復元。
- フィールド種別(username/password/totp)ごとにクリア対象を分岐させること。全フィールドを一律の対象とする。

## Decisions

### 1. 「直近にアプリが書き込んだクリップボード値」を保持する共有state (`ClipboardGuard`) を新設する

`backend/idle.rs` の `IdleTimer` と同じ、`Arc<Mutex<...>>` でラップした小さな構造体を `backend/clipboard_guard.rs`(仮)に新設する。

```rust
pub struct ClipboardGuard {
    last_written: Arc<Mutex<Option<String>>>,
}

impl ClipboardGuard {
    pub fn set(&self, value: String);
    pub fn clear(&self);
    // 現在のクリップボードの中身(current)が自分が書き込んだ値と一致するかどうかを
    // 判定する純粋ロジック。実際のクリップボードI/Oには触れない。
    pub fn should_clear(&self, current: &str) -> bool;
}
```

`copy_field` は書き込み時に `guard.set(value.clone())` を呼ぶ。`lock` 系の即時クリア、および遅延タスクは `guard.should_clear(current)` で判定してからクリアし、クリア後は `guard.clear()` で内部状態をリセットする(以後の誤判定・機微値の保持時間を最小化するため)。

**代替案として検討したもの:**
- 各 `copy_field` 呼び出しのクロージャに捕捉した値だけで比較し、共有stateを持たない案。遅延タスク単体では成立するが、`lock` 側から「直前にコピーされた値」を参照する手段がなくなるため、ロック時のクリアを実現できない。共有stateが必須と判断した。

### 2. 遅延クリアは `copy_field` 内で `tauri::async_runtime::spawn` により起動する

既存の `watch_idle_timeout`(常時ポーリングループ)とは異なり、クリアは「コピーの都度、一回だけ発火する」性質のタスクである。`copy_field` の最後で以下のようなfire-and-forgetタスクを起動する。

```rust
app.clipboard().write_text(value.clone())?;
guard.set(value.clone());

let app_for_clear = app.clone();
let guard_for_clear = guard.inner().clone();
tauri::async_runtime::spawn(async move {
    tokio::time::sleep(CLIPBOARD_CLEAR_DELAY).await;
    let Ok(current) = app_for_clear.clipboard().read_text() else { return; };
    if guard_for_clear.should_clear(&current) {
        let _ = app_for_clear.clipboard().write_text(String::new());
        guard_for_clear.clear();
    }
});
```

**代替案として検討したもの:**
- 世代カウンタ(コピーごとにインクリメントするID)を使い、遅延タスク発火時に「自分が最新のコピーか」を確認する方式。クリップボードの実際の中身を検証しないため、ユーザーがOSレベルで(アプリを介さず)別の値をコピーした場合を検出できない。issue本文が要求している「クリップボードの中身を検証してからクリアする」という要件を満たさないため不採用。

### 3. `lock`(手動)・`watch_idle_timeout`(アイドル自動)はロック成功後に同じ即時クリア処理を呼ぶ

両者から共通の関数(例: `clear_clipboard_if_owned(app, guard)`)を呼び出す形にし、判定ロジック(`ClipboardGuard::should_clear`)を再利用する。ロックの成否に関わらずクリア処理を行うと、ロック失敗時に不必要な副作用が発生するため、ロック成功後にのみ実行する。

### 4. 判定ロジックを純粋関数として切り出し、ユニットテスト可能にする

`ClipboardGuard::should_clear(&self, current: &str) -> bool` は内部の `Option<String>` との文字列比較のみで、`tauri::AppHandle` や実際のOSクリップボードに依存しない。`main.rs` の `extract_path_from_marker` と同じ設計方針で、`#[cfg(test)]` によるユニットテストを実装する。実際のクリップボードI/O(`app.clipboard().read_text()/write_text()`)を伴う統合的な動作確認は、既存の `copy_field`/`lock` と同様に手動確認の範囲とする(プロジェクトに `tauri::test::mock_app()` 等のモック基盤が存在しないため)。

## Risks / Trade-offs

- [遅延タスクの発火前にアプリが終了した場合、クリアが行われない(`tauri::async_runtime` のタスクはアプリプロセスと運命を共にする)] → 他のパスワードマネージャーでも同様の制約があり、許容する。
- [`ClipboardGuard` が機微値をメモリ上に保持する時間が(既存の `copy_field` 単体に比べ)わずかに伸びる] → クリア成功時に即座に `guard.clear()` で内部状態を破棄し、保持時間を最小化する。
- [クリップボードの読み取り(`read_text()`)が失敗するケース(例: 他アプリが排他ロックしている等)] → 読み取り失敗時は安全側に倒してクリアを行わない(誤って無関係な値を消すリスクを避ける)。

## Migration Plan

- 既存の `copy_field`/`lock`/`watch_idle_timeout` の呼び出しシグネチャや戻り値は変更しない。内部に処理を追加するのみで、呼び出し元(WebView側JS)への影響はない。
- 新規state (`ClipboardGuard`) は `tauri::Builder::manage()` で追加登録する。既存のstate登録(`AppState`/`IdleTimer`等)と同様のパターンのため、既存コードへの破壊的変更はない。
- ロールバックはコミットのrevertで対応可能。

## Open Questions

- 遅延時間30秒が実際の利用感として適切かは、実装後にissue報告者や利用者のフィードバックで見直す余地がある(現時点ではissueが提案する「30秒〜1分」の下限を採用)。

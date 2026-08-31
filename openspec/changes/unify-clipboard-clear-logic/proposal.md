## Why

クリップボードのクリア処理が `commands.rs` 内に2つの異なる実装パターンで存在している(issue #127)。`lock()` から呼ばれる共通関数 `clear_clipboard_if_owned` と、`copy_field` 内にインラインで書かれた遅延クリア処理は、目的(アプリが書き込んだ値のままなら安全にクリアする)が同じにもかかわらず、比較対象・クリア方法が異なる実装になっており、可読性・保守性を損なっている。

## What Changes

- `ClipboardGuard` に、現在保持している値のクローンを返す `last_value() -> Option<String>` アクセサを追加する。
- `clear_clipboard_if_owned` のシグネチャを `(app, guard, expected: &str)` に変更し、`guard.should_clear()` ではなく渡された `expected` とクリップボードの現在値を直接比較する方式に統一する。クリア成功時は `guard.clear_if_matches(expected)` を呼ぶ(無条件の `guard.clear()` は廃止)。
- `lock()` コマンドの呼び出し箇所を、`guard.last_value()` が `Some(v)` の場合のみ `clear_clipboard_if_owned(&app, &guard, &v)` を呼ぶ形に変更する。
- `copy_field` 内の遅延クリアのインライン実装(`tokio::time::sleep` 後の読み取り・比較・書き込みブロック)を削除し、`clear_clipboard_if_owned(&app_for_clear, &guard_for_clear, &expected_for_clear)` の呼び出しに置き換える。
- 単純な関数差し替えではなく `expected` を明示的に受け渡す設計にすることで、30秒以内に複数回コピーした場合に後発のコピー値が先発コピーのタイマーによって早期にクリアされてしまう回帰(guardの内部状態が上書きされることに起因する競合状態)を防ぐ。

このリファクタリングは既存の観測可能な振る舞い(クリップボードクリアのタイミング・条件)を変えないことを目的としており、`openspec/specs/credential-copy-actions/spec.md` が既に規定している30秒後クリア・上書き時スキップの要求はそのまま維持される。

## Capabilities

### New Capabilities
(なし)

### Modified Capabilities
(なし。仕様上の要求(`credential-copy-actions`)は変更せず、実装をその要求に整合させたまま重複コードを解消するのみ)

## Impact

- 影響ファイル: `app/src-tauri/src/commands.rs`, `app/src-tauri/src/backend/clipboard_guard.rs` の2ファイルのみ。
- `clear_clipboard_if_owned` のシグネチャ変更に伴い、既存の呼び出し箇所(`lock()`)も追随して修正が必要。
- 外部API・UI・依存関係への影響なし。
- 既存のユニットテスト(`clipboard_guard.rs`, `commands.rs`)を維持しつつ、上記の競合状態(連続コピー時の早期クリア防止)を検証する回帰テストを追加する。

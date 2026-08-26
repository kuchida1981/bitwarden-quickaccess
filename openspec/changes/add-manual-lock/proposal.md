## Why

GitHub issue #66: 現在、ユーザーが明示的にVaultをロックする手段がなく、`idle-auto-lock` によるアイドルタイムアウト(15分)を待つしかない。離席時などに即座にロックしたい場合の導線が無い。

バックエンド側は `BwServeClient::lock()`(`/lock` エンドポイント呼び出し)が既に実装済みで、これを呼び出すTauriコマンド・UI導線を追加するだけで実現できる。

## What Changes

- `commands.rs` に `lock` コマンドを追加する。`BwServeClient::lock()` 呼び出し成功後に `AppState::set_locked()` を反映する。
- 検索画面のショートカット `⌘L` でロックを実行できるようにする。ロック実行後、ポップアップが表示されていればアンロック画面に切り替える。
- トレイメニューに「今すぐロック」項目を追加する。アンロック済みの場合のみ有効化し、それ以外(ロック中・未接続)では無効化する。

## Capabilities

### New Capabilities
- `manual-lock`: ユーザー操作(検索画面のショートカット、トレイメニュー)による明示的なVaultロック機能。

### Modified Capabilities
- `menubar-presence`: 「コンテキストメニュー」要件に、明示的ロック項目が含まれることを追記する。

## Impact

- `app/src-tauri/src/commands.rs`: `lock` コマンドの追加
- `app/src-tauri/src/main.rs`: `invoke_handler!` への登録
- `app/src-tauri/src/tray.rs`: 「今すぐロック」メニュー項目の追加、クリック時の処理、ロック状態に応じた有効/無効切り替え
- `app/src-tauri/src/i18n.rs`: トレイメニュー用ラベル(`lock_now_label`)の追加(日英)
- `app/dist/app.js`: `⌘L` ショートカットの処理、ロック実行後のアンロック画面への切り替え
- `app/dist/index.html`: ヘルプオーバーレイへの `⌘L` の追記
- 破壊的変更なし。既存のアイドル自動ロックの挙動には影響しない。

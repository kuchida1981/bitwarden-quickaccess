## Why

`copy_field` でパスワードやTOTPをクリップボードにコピーした後、自動的にクリアする仕組みが存在せず、平文の機密情報がクリップボードに残り続ける([GitHub Issue #80](https://github.com/kuchida1981/bitwarden-quickaccess/issues/80))。他アプリからの読み取りや誤ペーストなど、情報漏洩のリスクがある。1Password や Bitwarden 公式アプリ等、一般的なパスワードマネージャーでは一定時間後の自動クリアが標準機能として提供されており、本アプリでも同等の体験を提供する必要がある。

## What Changes

- `copy_field`(username/password/totp)でクリップボードに書き込んだ値を、30秒後に自動的にクリアする。ただし、その時点でクリップボードの中身が書き込んだ値のままである場合のみクリアし、ユーザーが別の値をコピー済みの場合は上書き・誤消去しない。
- 手動ロック(`⌘L`・トレイの「今すぐロック」)実行時にも、クリップボードの中身が直前にアプリが書き込んだ値のままであれば即座にクリアする。
- アイドルタイムアウトによる自動ロック実行時にも、同様に即座にクリアする。
- 上記の「クリップボードの中身が期待値のままか」を判定する共有状態(`ClipboardGuard`)を新設し、コピー操作とロック操作の双方から参照できるようにする。

## Capabilities

### New Capabilities

(なし)

### Modified Capabilities

- `credential-copy-actions`: コピー実行後、一定時間(30秒)でクリップボードを自動クリアする要件を追加する。
- `manual-lock`: 手動ロック実行時にクリップボードを即座にクリアする要件を追加する。
- `idle-auto-lock`: アイドルタイムアウトによる自動ロック実行時にクリップボードを即座にクリアする要件を追加する。

## Impact

- `app/src-tauri/src/commands.rs`: `copy_field`(クリア用遅延タスクの起動・guardへの書き込み)、`lock`(ロック成功後の即時クリア呼び出し)
- `app/src-tauri/src/main.rs`: `watch_idle_timeout`(アイドル自動ロック成功後の即時クリア呼び出し)
- `app/src-tauri/src/backend/`: 新規モジュール(`ClipboardGuard` 相当の共有state、`backend/idle.rs` と同様のMutexベースのパターン)を追加
- 新規依存クレートの追加は不要(既存の `tauri-plugin-clipboard-manager` / `tokio` の time機能を利用)

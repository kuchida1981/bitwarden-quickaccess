## Why

GitHub issue #79: `bw`(Bitwarden CLI)が未インストール・`bw serve`未対応バージョン・未ログイン等の理由でバックエンド接続に失敗した場合、原因がユーザーに一切伝わらない。

現状の挙動:
1. 起動時のpreflightチェック(`preflight::check_bw_cli()`)や初回status同期(`sync_initial_status()`)が失敗すると、`AppState::set_error()` にエラーメッセージが記録される(`AppState::last_error()` として既に取得可能)が、これを問い合わせるフロントエンド向けコマンドが存在しない。
2. バックエンド状態は `BackendState::Disconnected` のままだが、`get_lock_state` コマンドは `"disconnected"` を返すのみで、フロントエンド(`handleShown()`)はこれを `"unlocked"` 以外として扱うため、**アンロック画面(マスターパスワード入力フォーム)が誤って表示される**。
3. ユーザーがパスワードを入力して送信すると、ポート未設定のため「バックエンドサービスの準備がまだできていません。」という間接的で原因の分からないエラーが表示される。

## What Changes

- `AppState::last_error()` を取得する新しいTauriコマンド(`get_backend_error`)を追加する。
- フロントエンドに専用のエラー画面(`#error-screen`)を追加する。
- `handleShown()` の画面判定を、`lockState` の3値(`"unlocked"` → 検索画面 / `"locked"` → アンロック画面 / `"disconnected"` → エラー画面)に応じて正しく分岐するよう修正する。
- エラー画面表示時、`get_backend_error` で取得した具体的なエラーメッセージ(`bw`未検出、`bw serve`未対応、未ログイン等)をそのまま表示する。

## Capabilities

### New Capabilities
- `backend-connection-error-display`: バックエンド接続(`bw` CLI前提条件・`bw serve`起動・ログイン状態)に問題がある場合、原因を示す専用エラー画面を検索ポップアップに表示する機能。

### Modified Capabilities
(なし。`vault-unlock-prompt` はロック中の正常系フローを扱うため変更不要。`disconnected` 状態は新capabilityとして切り出す)

## Impact

- `app/src-tauri/src/commands.rs`: `get_backend_error` コマンドの追加
- `app/src-tauri/src/main.rs`: `invoke_handler!` への登録
- `app/dist/index.html`: `#error-screen` セクションの追加
- `app/dist/app.js`: `showScreen()`, `handleShown()` の3値分岐対応
- `app/dist/i18n.js`: エラー画面の見出し文言(日英)の追加
- `app/dist/style.css`: `#error-screen` のレイアウト(中央寄せ・余白)を `#unlock-screen` に合わせて追加
- `app/src-tauri/src/backend/process.rs`: `bw serve` プロセスがセッション中に予期せずクラッシュした場合も `last_error` にメッセージを記録するよう修正(コードレビューで発覚)
- 破壊的変更なし。`get_lock_state` の既存の戻り値・呼び出し元への影響なし(新規コマンド追加のみ)。

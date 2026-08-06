## Why

Bitwarden の認証情報を使うたびに Web Vault やデスクトップアプリを開いて検索するのは遅い。1Password の Quick Access のように、ターミナルからホットキー感覚でアイテムを絞り込み、ユーザー名・パスワード・TOTP を即座にクリップボードへコピーできれば、日常的な認証情報の取り出し体験が大きく速くなる。`bw` CLI・`jq`・`fzf` という既存の UNIX ツールを組み合わせれば、専用アプリを作らずにこれを実現できる。

## What Changes

- 新規ターミナルツール `bw-quickaccess`(仮称)を追加し、以下を提供する:
  - `bw` CLI から取得した vault アイテムを `fzf` でインクリメンタルサーチできる検索画面
  - アイテム選択後、ユーザー名・パスワード・TOTP を選んでクリップボードへコピーできるフィールド選択画面。選択してもすぐには終了せず、Esc 等の明示操作まで同じアイテムの別フィールドを連続コピーできる
  - 直前に選択したアイテムをローカルにキャッシュし、次回起動時は検索画面をスキップしてそのアイテムのフィールド選択画面から始められる(Esc で検索画面に戻れる)
  - `bw unlock` で得た session token を OS keychain(macOS Keychain / Linux Secret Service)にキャッシュし、TTL 内はマスターパスワード入力を要求しない
  - 必須の外部コマンド(`bw` / `jq` / `fzf` / クリップボードコピーコマンド)が未導入の場合は、インストール方法を案内した上でエラー終了する
  - パスワード・TOTP 等の機密情報を標準出力へ一切出力しない(クリップボード経由のみ)
- 以下は明示的にスコープ外とする(検討の上、見送り):
  - Bitwarden デスクトップアプリで選択アイテムを直接開く連携(deep link 未対応のため)
  - クリップボードの自動クリア
  - デスクトップアプリのロック状態の横取り
  - Linux のヘッドレス/SSH 専用環境のサポート(デスクトップ GUI 環境が前提)

## Capabilities

### New Capabilities
- `vault-item-search`: `bw` から取得した vault アイテムを `fzf` でインクリメンタルサーチする検索画面、および直前選択アイテムを記憶して検索をスキップする挙動
- `credential-clipboard-copy`: アイテムのユーザー名・パスワード・TOTP をフィールド選択画面から選んでクリップボードへコピーする機能(連続コピー対応、機密情報を標準出力に出さない制約を含む)
- `bw-session-management`: `bw unlock` の session token を OS keychain にキャッシュ・TTL 管理し、実行のたびのマスターパスワード入力を避ける仕組み
- `environment-preflight`: 起動時に必須外部コマンド(`bw`/`jq`/`fzf`/クリップボードコピーコマンド)の存在を確認し、不足時はインストール案内をしてエラー終了する仕組み

### Modified Capabilities
(なし。リポジトリは初期化直後で既存 spec は存在しない)

## Impact

- 新規リポジトリへの新規ツール追加であり、既存コードへの影響はない
- 外部依存: `bw`(Bitwarden CLI)、`jq`、`fzf`、macOS の `pbcopy`/`security`、Linux の `xclip`/`xsel`/`wl-copy` と `secret-tool`(libsecret)
- 対応 OS: macOS、Linux(デスクトップ GUI 環境)

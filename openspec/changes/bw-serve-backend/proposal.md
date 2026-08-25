## Why

`bw-quickaccess` を 1Password Quick Access 相当のメニューバー常駐GUIアプリに置き換えるにあたり(関連change群: `menubar-hotkey-shell` / `quickaccess-search-ui` / `credential-actions-autolock` / `gui-distribution-and-tui-removal`)、まず vault へのアクセスを担う土台が必要になる。現行の `bin/bw-quickaccess` は `bw` コマンドをキー入力のたびにサブプロセスとして都度起動する設計だが、常駐GUIアプリではこの使い捨てパターンは非効率であり、ホットキー押下からポップアップ表示までの体感速度にも悪影響がある。

`bw` CLI には `bw serve` という公式機能があり、vaultをアンロックした状態でメモリに保持したまま localhost REST API として待ち受けられる。認証・vault復号・TOTP計算といったセキュリティ上リスクの高い処理を独自実装せずBitwarden公式実装に完全委譲できるため、この上にTauri(Rust)製の常駐バックエンドを構築する。

## What Changes

- 新規に Tauri(Rust)プロジェクトを `app/` 以下に作成する(UIは持たない最小限の雛形。ウィンドウ・トレイ・ホットキーは後続changeで追加)。
- `bw serve` を子プロセスとして起動・監視するモジュールを実装する。
  - 起動時に空きポートを動的に選択し(固定ポート衝突回避)、`localhost` にのみバインドする。
  - 子プロセスの異常終了を検知し、再起動またはエラー通知を行う。
  - アプリ終了時に子プロセスを確実に終了(kill)させる。
- `bw serve` の HTTP API に対する薄いRustクライアントラッパーを実装する(`status` / `unlock` / `lock` / `list items(search)` / `get item` / `get totp`)。
- vault のロック状態(`locked` / `unlocked`)をRust側で保持し、後続changeのUI層から参照できる形にする。
- 既存の `bin/bw-quickaccess` および `lib/*.sh` には変更を加えない(参照実装として残すのみ。削除は `gui-distribution-and-tui-removal` のスコープ)。

## Capabilities

### New Capabilities
- `vault-backend-service`: `bw serve` プロセスのライフサイクル管理と、HTTP経由でのvault操作(状態取得・アンロック・ロック・アイテム検索・TOTP取得)を提供する内部バックエンドサービス。

### Modified Capabilities
(なし。既存specsは全て現行TUI向けの挙動を記述しており、本changeはUIを持たない内部サービスの追加のみのため、既存の観測可能な挙動に変更はない。)

## Impact

- 新規依存: Rust toolchain, Tauri CLI, Node.js(Tauriのフロントエンドビルド基盤として)。
- 新規ディレクトリ `app/`(Tauriプロジェクト一式)。
- 既存の `bin/bw-quickaccess` / `lib/*.sh` / 既存テスト(`test/lib/*.bats`)には影響しない。
- CI: 本changeでは `app/` 向けのCI追加は最小限(ビルド確認程度)に留め、本格的なCI整備は後続change(`gui-distribution-and-tui-removal` 等)で検討する。
- ユーザー向け影響: なし(本changeはUIを持たないため、エンドユーザーの操作フローに変化はない)。

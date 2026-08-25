## Why

`bw-serve-backend` / `menubar-hotkey-shell` / `quickaccess-search-ui` / `credential-actions-autolock` により、GUIアプリが現行TUI(`bin/bw-quickaccess`)の主要機能(検索・ユーザー名/パスワード/TOTPコピー・ブラウザで開く)を代替できる状態になった。v1.0.0のマイルストーンとしてTUIを完全に廃止しGUIに一本化する方針(既定)に基づき、旧TUIのコード・テスト・配布経路を削除し、新しい配布方法(セルフビルド + 未署名GitHub Releases)とドキュメントに置き換える。

## What Changes

- `bin/bw-quickaccess` / `lib/*.sh` / `install.sh` / `test/lib/*.bats` を削除する(**BREAKING**: 既存のcurlワンライナーによるインストール、および `bw-quickaccess` コマンド自体が使えなくなる)。
- `README.md` / `README.ja.md` をGUIアプリ前提の内容に全面書き換えする(必要要件、セルフビルド手順、GitHub Releasesからのインストール手順、使い方=ホットキー・アクション一覧)。
- `.github/workflows/ci.yml` を、bashの構文チェック・shellcheck・bats実行から、Tauri/Rustのビルド・テストに置き換える。
- `.github/workflows/release.yml` を、macOS向け `.app` のビルドと未署名アセットのGitHub Releasesへの自動添付に置き換える(既存のTUI向けバンドル・i18nメッセージファイル同梱ロジックは削除する)。
- コード署名・notarizationは引き続き本changeのスコープ外とする(将来必要になれば別途change化する)。

## Capabilities

### New Capabilities
- `gui-distribution`: macOS向けGUIアプリのセルフビルド手順(`git clone` + `tauri build`)と、未署名 `.app` のGitHub Releasesを通じた配布・更新手順を提供する。

### Modified Capabilities
以下は全て現行TUI固有の挙動を記述したcapabilityであり、TUI廃止に伴い全要件を廃止(REMOVED)する。後継の挙動は `bw-serve-backend` / `menubar-hotkey-shell` / `quickaccess-search-ui` / `credential-actions-autolock` の各capabilityが担う。

- `bw-session-management`: session管理はGUIアプリの `vault-backend-service` / `idle-auto-lock` に置き換わる。
- `copy-feedback`: コピー結果フィードバックはGUIアプリの `credential-copy-actions` に置き換わる。
- `credential-clipboard-copy`: クリップボードコピー機能はGUIアプリの `credential-copy-actions` / `open-in-browser-action` に置き換わる。
- `environment-preflight`: 起動時の依存チェックはGUIアプリの `vault-backend-service`(bw CLI前提チェック)に置き換わる。`jq`/`fzf`/OSクリップボードコマンドはGUIアプリでは不要になるため要件ごと廃止する。
- `installation`: curlワンライナーによるインストールは `gui-distribution` に置き換わる。
- `loading-feedback`: ローディング表示はGUIアプリの `vault-unlock-prompt` 等に置き換わる。
- `message-localization`: TUIの `LANG`/`BWQA_LANG` によるメッセージ言語切替は廃止する。GUIアプリでのローカライズは本changeのスコープ外(non-goal)とし、必要になれば将来のchangeで再導入する。
- `release-packaging`: 単一ファイルへのバンドル・リリース添付ロジックは `gui-distribution` に置き換わる。
- `test-automation`: bats/shellcheckによるテスト自動化は廃止する。Tauri/Rustコードベースのテスト戦略は実装時に別途定める。
- `vault-item-search`: fzfベースの検索画面は `incremental-item-search` に置き換わる。

## Impact

- 削除: `bin/bw-quickaccess`, `lib/*.sh`, `install.sh`, `test/lib/*.bats`, `test/helpers/*.bash`
- 変更: `README.md`, `README.ja.md`, `.github/workflows/ci.yml`, `.github/workflows/release.yml`
- **BREAKING**: 既存ユーザーの `bw-quickaccess` コマンド・curlインストール導線が使えなくなる。README刷新が実質的な移行案内を兼ねる。
- 本changeの完了をもってv1.0.0のマイルストーンが達成される。

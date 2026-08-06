## Why

`bin/bw-quickaccess` と `lib/*.sh` には自動テストが一切無く、CI(GitHub Actions 等)も未整備。動作確認は手動でのみ実施されており(過去の change でも 6.1-6.5 は手動検証として記録されている)、リグレッションの検知がレビュー者の目視に依存している。GitHub issue #6 で、外部コマンド(`bw`/`fzf`/`security`/`secret-tool`)をモックした単体テストと、それを回す CI の整備が要望されている。

## What Changes

- `lib/common.sh`, `lib/session.sh`, `lib/search.sh`, `lib/fields.sh`, `lib/preflight.sh` の純粋ロジック・分岐ロジックを対象に、`bats-core` による単体テストを追加する
  - 外部コマンド呼び出しは関数スタブ(`bwqa_bw()` のオーバーライド等)を中心にモックする
  - `bwqa_check_core_tools` / `bwqa_check_fzf_version` / clipboard コマンドなど、コマンド有無・バージョン検出そのものが検証対象の箇所は PATH 上のダミー実行ファイルでモックする
  - `bw unlock` や実際の vault アクセスを伴う結合テスト(`bwqa_unlock`/`bwqa_get_session` の実利用パス)はスコープ外とする
- GitHub Actions ワークフロー(`.github/workflows/ci.yml`)を新規追加し、`macos-latest` / `ubuntu-latest` の matrix で `bash -n` 構文チェック・`shellcheck` 静的解析・`bats` テスト実行を回す
- README.md にテスト実行方法(ローカルでの `bats-core`/`shellcheck` セットアップ含む)を追記する

## Capabilities

### New Capabilities
- `test-automation`: `lib/*.sh` の純粋ロジック・分岐ロジックに対する bats-core 単体テスト、および GitHub Actions による構文チェック・静的解析・テスト実行の自動化

### Modified Capabilities
(なし。既存の `credential-clipboard-copy` / `environment-preflight` / `vault-item-search` / `bw-session-management` の振る舞い要件そのものは変更しない。テストは既存要件を検証する手段であり、要件自体の変更ではない)

## Impact

- 追加: `test/` ディレクトリ(bats テストファイル、fixtures、テストヘルパー)、`.github/workflows/ci.yml`
- 変更: `README.md`(開発者向けセクションにテスト実行方法を追記)
- 依存追加: `bats-core`(ローカル: `brew install bats-core` 等、CI: パッケージマネージャ経由)、`shellcheck`(CI: Ubuntu ランナーに標準搭載、macOS ランナーは `brew install shellcheck`)
- 既存の `bin/bw-quickaccess` / `lib/*.sh` の実装ロジック自体への変更は想定しない(テスト容易化のための最小限のリファクタが必要になった場合は design.md に明記する)

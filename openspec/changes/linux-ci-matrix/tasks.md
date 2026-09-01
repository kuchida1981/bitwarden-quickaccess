## 1. CI ワークフローの更新

- [ ] 1.1 `.github/workflows/ci.yml` で `jobs.test` に `strategy.matrix.os: [macos-latest, ubuntu-latest]` を設定し、`runs-on: ${{ matrix.os }}` に変更する
- [ ] 1.2 Linux 環境用（`if: runner.os == 'Linux'`）のシステム依存パッケージインストールステップ（WebKitGTK, AppIndicator, RSVG 等）を追加する

## 2. 構文・フォーマット・CI 実機検証

- [ ] 2.1 YAML 構文および GitHub Actions 定義に誤りがないことを確認する
- [ ] 2.2 既存の step（fmt, build, test, clippy）が両 OS で動作する記述になっていることを確認する
- [ ] 2.3 PR 作成後に GitHub Actions で `macos-latest` と `ubuntu-latest` の両マトリクスが実際にパスすることを確認する

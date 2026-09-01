## Why

#145 によりバックエンドが Linux でビルド・テスト可能になりましたが、現在の GitHub Actions CI (`.github/workflows/ci.yml`) は `macos-latest` のみで実行されているため、今後の変更で Linux 環境でのビルドやテストのデグレを自動検知できません（Issue #149 CI パート）。
Linux サポートを継続的に保証するため、CI ワークフローに `ubuntu-latest` を追加し、マルチプラットフォーム（macOS + Linux）での自動検証を開通させる必要があります。

## What Changes

- `.github/workflows/ci.yml` の `test` ジョブをマトリクス化（`matrix.os: [macos-latest, ubuntu-latest]`）
- Linux 環境（`runner.os == 'Linux'`）に必要なシステムライブラリ（`libwebkit2gtk-4.1-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev` 等）を `apt-get` でインストールするステップを追加
- Linux / macOS 双方で `cargo fmt`, `cargo build`, `cargo test`, `cargo clippy` を一貫して実行・検証

## Capabilities

### New Capabilities

### Modified Capabilities

## Impact

- **CI ワークフロー**: `.github/workflows/ci.yml`
- **対象環境**: GitHub Actions 上で `ubuntu-latest` と `macos-latest` の両方で CI が実行されるようになる

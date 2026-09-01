## Context

現在 `.github/workflows/ci.yml` では `runs-on: macos-latest` のみで CI を実行しています。
Issue #145 のマージにより Rust バックエンドが Linux でもビルド・テストできるようになりました。本 change では CI をマルチプラットフォーム（macOS + Linux）化し、継続的テスト環境を構築します。

## Goals / Non-Goals

**Goals:**
- `.github/workflows/ci.yml` において、`macos-latest` および `ubuntu-latest` のマトリクスでテストを実行する
- Linux ランナー上で Tauri v2 のビルド・テストに必要な apt パッケージを自動インストールする
- `cargo fmt`, `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings` が両 OS でパスすることを確認する

**Non-Goals:**
- Release ワークフロー（`.github/workflows/release.yml`）の Linux パッケージング追加（Issue #148 完了後のリリース自動化で実施）
- Linux 固有の UI / E2E テストの追加

## Decisions

### 1. マトリクス構成
- **決定**: `jobs.test` に `strategy.matrix.os: [macos-latest, ubuntu-latest]` を設定し、`runs-on: ${{ matrix.os }}` とする。
- **理由**: 単一のジョブ定義で steps を共通化し、OS 固有のステップのみ `if: runner.os == 'Linux'` で分岐することで保守性を高めるため。

### 2. Linux 依存パッケージのインストール
- **決定**:
  ```yaml
  - name: Install Linux system dependencies
    if: runner.os == 'Linux'
    run: |
      sudo apt-get update
      sudo apt-get install -y libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev patchelf libxdo-dev libssl-dev
  ```
- **理由**: Tauri v2、トレイ・アイコン機能、およびグローバルショートカット／クリップボード連携プラグイン（`tauri-plugin-global-shortcut`, `tauri-plugin-clipboard-manager`）に必要なライブラリを確実に導入するため。

### 3. キャッシュ設定
- **決定**: `Swatinem/rust-cache@v2` は OS ごとに自動でキーが分離されるため、既存のキャッシュ設定をそのまま維持する。

## Risks / Trade-offs

- **[Risk]** Linux 環境での `apt-get` やビルドにより CI 実行時間が若干増加する。
  → **Mitigation**: `rust-cache` によるビルドアーティファクトのキャッシュにより、2回目以降のビルド時間は最小化される。

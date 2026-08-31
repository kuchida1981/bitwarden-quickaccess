## Why

CI ワークフロー (`ci.yml`) では `cargo build` / `cargo test` / `cargo clippy` は実行されているが、`cargo fmt --check` が含まれていない。フォーマットが崩れたコードでもマージを妨げられず、レビューでの指摘やレビュアーごとの表記ゆれの原因になりうる (issue #120)。

現状の `app/src-tauri` 配下を `cargo fmt --check` にかけると、12ファイル・53箇所で未フォーマットの差分が検出される。ステップを追加するだけでは既存コードが理由で CI が恒常的に赤くなるため、ステップ追加と既存コードの一括フォーマットを同じ change で行う必要がある。

## What Changes

- `ci.yml` に `cargo fmt --check` ステップを追加する(`cargo clippy` と並列に実行できる独立ステップとして)
- 既存の `app/src-tauri` 配下のコードに `cargo fmt` を適用し、フォーマット崩れを解消する
- フォーマット適用後も `cargo build` / `cargo test` / `cargo clippy --all-targets -- -D warnings` が通ることを確認する

## Capabilities

このchangeはCIパイプラインの構成変更のみであり、アプリケーションの振る舞い(spec)は変更しない。`skip_specs: true` を設定済み。

### New Capabilities

なし

### Modified Capabilities

なし

## Impact

- `.github/workflows/ci.yml`: `cargo fmt --check` ステップを追加
- `app/src-tauri/build.rs` および `app/src-tauri/src/**/*.rs` の対象ファイル(フォーマット適用による整形のみ、ロジック変更なし):
  - `build.rs`
  - `src/backend/clipboard_guard.rs`
  - `src/backend/http_client.rs`
  - `src/backend/idle.rs`
  - `src/backend/preflight.rs`
  - `src/backend/process.rs`
  - `src/backend/state.rs`
  - `src/commands.rs`
  - `src/i18n.rs`
  - `src/main.rs`
  - `src/popup.rs`
  - `src/tray.rs`

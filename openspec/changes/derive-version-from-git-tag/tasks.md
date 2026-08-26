## 1. ビルド時のバージョン導出

- [x] 1.1 `app/src-tauri/build.rs` を修正し、`tauri_build::build()` に加えて以下を行う: `CARGO_MANIFEST_DIR` を起点に、`git describe --tags --always` を実行し、成功すればその標準出力(トリム済み)を、失敗(gitコマンドが無い/`.git`が無い等)すれば `format!("v{}", env!("CARGO_PKG_VERSION"))` を、`cargo:rustc-env=BWQA_DISPLAY_VERSION=<値>` として出力する。
- [x] 1.2 同じ `build.rs` に、`cargo:rerun-if-changed=<repo_root>/.git/HEAD` と `cargo:rerun-if-changed=<repo_root>/.git/refs/tags` を追加し、タグ変更時に再ビルドが走るようにする(`repo_root` は `CARGO_MANIFEST_DIR` から2階層上)。
- [x] 1.3 `app/src-tauri/src/tray.rs` の `const APP_VERSION: &str = env!("CARGO_PKG_VERSION");` を `const APP_VERSION: &str = env!("BWQA_DISPLAY_VERSION");` に変更する。
- [x] 1.4 同ファイルの `about_item` 組み立て箇所 `format!("{} v{}", app.package_info().name, APP_VERSION)` を `format!("{} {}", app.package_info().name, APP_VERSION)` に変更する(`APP_VERSION` が既に `v` を含むため二重にならないようにする)。

## 2. リリースワークフローの簡素化

- [x] 2.1 `.github/workflows/release.yml` から「Sync Cargo.toml version with the release tag」ステップを削除する。
- [x] 2.2 同ファイルの `actions/checkout@v4` ステップに `fetch-depth: 0` を追加し、タグ情報を確実に取得できるようにする。

## 3. ドキュメント更新

- [x] 3.1 `CONTRIBUTING.md` のリリース手順から、「Cargo.tomlはビルド用チェックアウト内でのみタグから同期される」という記述を、「バージョンはビルド時に`git describe`から動的に導出されるため、`Cargo.toml`の更新もCI側での同期も一切不要」という趣旨に更新する。

## 4. 動作確認

- [ ] 4.1 `cargo test` を実行し、既存テストが通ることを確認する。
- [ ] 4.2 実機で、現在のリポジトリ状態(最新タグより後のコミット)で `cargo run` し、トレイメニューのバージョン表示が `v1.1.0-N-gXXXXXXX` のような形式になることを確認する。
- [ ] 4.3 実機で、`git checkout <最新タグ>` した状態で `cargo run` し、トレイメニューのバージョン表示がクリーンな `vX.Y.Z` になることを確認する(確認後、元のブランチに戻すこと)。
- [ ] 4.4 `.github/workflows/release.yml` の変更内容をレビューし、次回の実リリース時に問題なく動作することを設計上確認する(実際のリリースは別途実施)。

## 1. Cargo 依存関係の分離

- [x] 1.1 `app/src-tauri/Cargo.toml` において、`objc2-app-kit` を macOS 専用の `[target.'cfg(target_os = "macos")'.dependencies]` に移動する

## 2. 最前面アプリ制御のプラットフォーム分離

- [x] 2.1 `app/src-tauri/src/popup.rs` で `objc2_app_kit` の import および `PreviousFrontmostApp`, `record_frontmost_app`, `restore_previous_focus` を macOS と非 macOS で `#[cfg]` 分岐する
- [x] 2.2 非 macOS 環境向けに no-op 実装を提供し、型シグネチャを維持する

## 3. ビルド・テスト検証

- [x] 3.1 Linux 環境で `cargo check --manifest-path app/src-tauri/Cargo.toml` が正常に通ることを確認する
- [x] 3.2 Linux 環境で `cargo test --manifest-path app/src-tauri/Cargo.toml` が全テストパスすることを確認する
- [x] 3.3 `cargo clippy` および `cargo fmt` を実行し、リグレッションがないことを確認する

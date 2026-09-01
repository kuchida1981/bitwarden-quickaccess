## Why

現在、Rust バックエンドには macOS 固有のクレート（`objc2-app-kit`）および macOS 固有の API 呼び出しが含まれており、Linux 環境でコンパイル・ビルド・テストを実行できません（Issue #145）。
マイルストーン v2.0.0 の最優先目標である Linux サポートおよび CI での継続的検証（#149）を実現するため、macOS 依存コードを分離・抽象化し、Linux 環境でのビルド・テストを可能にする必要があります。

## What Changes

- `Cargo.toml` において、`objc2-app-kit` 等の macOS 固有依存を `[target.'cfg(target_os = "macos")'.dependencies]` に移動
- `src/popup.rs` 内の最前面アプリ追跡・復帰処理（`PreviousFrontmostApp`, `record_frontmost_app`, `restore_previous_focus`）をプラットフォーム別に分離（macOS 向け NSWorkspace 実装と、非 macOS 向け no-op 実装）
- `src/main.rs` およびバックエンド全体のプラットフォーム依存呼び出しのガード確認
- Linux 環境における `cargo check`, `cargo build`, `cargo test` の開通確認

## Capabilities

### New Capabilities

### Modified Capabilities
- `quickaccess-window-focus`: 非 macOS (Linux 等) 環境において、最前面アプリのフォーカス復帰処理が no-op（非破壊的スキップ）として動作するプラットフォーム差異の要件を明確化。

## Impact

- **依存関係**: `Cargo.toml` の `dependencies` から `objc2-app-kit` が除外され、macOS ターゲット専用セクションへ移動
- **コードベース**: `app/src-tauri/src/popup.rs`, `app/src-tauri/src/main.rs`
- **対象環境**: Linux (x86_64, aarch64 等) でのビルドおよびテストが可能になる

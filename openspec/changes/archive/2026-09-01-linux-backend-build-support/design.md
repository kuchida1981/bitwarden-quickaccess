## Context

`bw-quickaccess-gui` は現在 macOS を主ターゲットとして開発されており、Rust バックエンドに macOS 固有のクレート（`objc2-app-kit`）や API 呼び出し（`set_activation_policy`）が直接含まれています。
マイルストーン v2.0.0 では Linux 正式対応を掲げており、その第1ステップとしてバックエンドのコンパイルおよびテストが Linux 上で正常に通る状態（Issue #145）を実現します。

## Goals / Non-Goals

**Goals:**
- `app/src-tauri` が Linux (x86_64 / aarch64 等) 環境で `cargo check`, `cargo build`, `cargo test` を完走できるようにする
- macOS 固有の依存・ロジックを `#[cfg(target_os = "macos")]` で適切に分離し、macOS 既存機能に一切デグレを発生させない
- 非 macOS 環境において `PreviousFrontmostApp` を型安全かつシームレスに扱える no-op 構造を提供する

**Non-Goals:**
- Linux (X11 / Wayland) における高度な最前面ウィンドウ取得・フォーカス復帰の実装（Linux ではデスクトップ環境ごとの差異が大きくセキュリティ制限もあるため、本 change では no-op とし、将来の課題とする）
- Linux 向けデスクトップパッケージング（deb/AppImage 等）の設定（Issue #148 で実施）
- GitHub Actions CI/CD の Linux ジョブ追加（Issue #149 で実施）
- フロントエンドのキーバインド（Ctrl/Cmd）切り替え（Issue #146 で実施）

## Decisions

### 1. `Cargo.toml` のプラットフォーム依存設定
- **決定**: `objc2-app-kit` を `[dependencies]` から `[target.'cfg(target_os = "macos")'.dependencies]` に移動する。
- **理由**: Linux ビルド時に macOS 固有の `objc2` がコンパイルエラーになるのを防ぐため。

### 2. `popup.rs` のフォーカス追跡のプラットフォーム分離
- **決定**: `PreviousFrontmostApp` 構造体と `record_frontmost_app`, `restore_previous_focus` を `cfg` フラグで切り替える。
  - macOS の場合: 現行通り `NSWorkspace` と PID によるフォーカス復帰を行う。
  - 非 macOS (Linux 等) の場合: `PreviousFrontmostApp` はダミー実装（空の struct または同等）とし、`record_frontmost_app` および `restore_previous_focus` は何もしない（no-op）。
- **理由**: `main.rs` での `manage(popup::PreviousFrontmostApp::new())` や `toggle_popup` 内での呼び出しコードを `#[cfg]` で汚さず、シグネチャと責務を透過的に保つため。

### 3. `main.rs` のプラットフォーム分離の確認
- **決定**:
  - `set_activation_policy` は既に `#[cfg(target_os = "macos")]` が適用されていることを維持。
  - `tauri_plugin_autostart` などの他の初期化はクロスプラットフォームで安全に動作することを確認。

## Risks / Trade-offs

- **[Risk]** Linux 環境でポップアップを閉じた際、直前のウィンドウにフォーカスが自動復帰しない。
  → **Mitigation**: Linux ではウィンドウマネージャ / コンポジタの標準挙動（ポップアップ消去時に自動で下層ウィンドウへフォーカスが移る等）に委ねる。

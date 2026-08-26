# bw-quickaccess-gui

1Password Quick Access 相当のメニューバー常駐GUIアプリ(Tauri/Rust製)。トップレベルの [README.md](../README.md) / [README.ja.md](../README.ja.md) にユーザー向けのインストール・使い方を記載している。このファイルは開発者向けの補足。

## 開発環境セットアップ

- Rust toolchain(stable。`rustup update stable` で最新化してください。Tauri v2の依存クレートが Rust 2024 edition を要求するため、古いツールチェーンでは `cargo build` が失敗します)
- Tauri CLI(配布用の `.app` をビルドする場合のみ必要): `cargo install tauri-cli --locked`
- Bitwarden CLI (`bw`)。`bw serve` サブコマンドに対応したバージョンが必要(`bw login` でログイン済みであること。ロック状態で構いません)

## ビルド・実行

```bash
cd app/src-tauri
cargo build
cargo run          # 開発時: アプリをその場で実行する
cargo test
cargo clippy --all-targets
cargo tauri build  # 配布用の .app バンドルを生成する(target/release/bundle/macos/ 配下)
```

## ディレクトリ構成

- `src-tauri/src/backend/` — `bw serve` プロセス管理・HTTPクライアント・状態管理(`cargo test` でテストされるlibクレート)
- `src-tauri/src/` 直下(`main.rs`/`commands.rs`/`tray.rs`/`popup.rs`/`hotkey.rs`) — Tauriアプリ本体(トレイ・ホットキー・ウィンドウ・IPCコマンド)
- `dist/` — WebViewのフロントエンド(素のHTML/CSS/JS、ビルドチェーンなし)

## 前提

- macOSのみ(v1.0.0スコープ)。Linux対応は将来のリリースで別途検討する。
- `bw` にログイン済みであること(`bw login` は本アプリの対象外)。

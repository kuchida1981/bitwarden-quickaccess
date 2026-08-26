# bw-quickaccess-gui (開発中)

1Password Quick Access 相当のメニューバー常駐GUIアプリ。現時点では `bw serve` の起動・監視とHTTPクライアントのみを持つバックエンド専用の雛形で、ウィンドウ・トレイ・グローバルホットキーは未実装(`menubar-hotkey-shell` change で追加予定)。

## 開発環境セットアップ

- Rust toolchain(stable, 1.85以降。`rustup update stable` で最新化してください。Tauri v2の依存クレートが Rust 2024 edition を要求するため、古いツールチェーンでは `cargo build` が失敗します)
- Node.js(将来のフロントエンドビルド用。現時点では `dist/` にプレースホルダのHTMLのみ配置)
- Bitwarden CLI (`bw`)。`bw serve` サブコマンドに対応したバージョンが必要(`bw login` でログイン済みであること。ロック状態で構いません)

## ビルド・実行

```bash
cd app/src-tauri
cargo build
cargo run    # bw serve を子プロセスとして起動する(ウィンドウは表示されない)
cargo test
```

## 動作確認済みの前提

- macOSのみ(v1.0.0スコープ)。Linux対応はv1.1.0で別途検討する。
- `bw` にログイン済みであること(`bw login` は本アプリの対象外)。

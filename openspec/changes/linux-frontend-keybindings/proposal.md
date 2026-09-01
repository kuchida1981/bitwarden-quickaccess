## Why

現在、フロントエンド（HTML / JS / i18n）は macOS の `⌘` (Cmd / `metaKey`) 前提で実装されています（Issue #146）。
Linux 環境では一般的なモディファイアキーである `Ctrl` (`ctrlKey`) を用いた操作が期待されるため、実行 OS に応じてキーイベント判定を切り替えるとともに、フッター、アクションメニュー、ヘルプオーバーレイにおけるショートカット表記を動的に `Ctrl` 表記に切り替える必要があります。

## What Changes

- Rust バックエンドに実行プラットフォームを返す Tauri コマンド `get_platform` を新設
- フロントエンド初期化時にプラットフォーム（`macos` / `linux` 等）を取得し、モディファイアキー判定（macOS: `metaKey`, Linux: `ctrlKey`）を動的に切り替え
- コピー（`KeyC`）、手動ロック（`KeyL`）、ヘルプ開閉（`Slash`）のキーイベントハンドラをマルチプラットフォーム対応化
- 検索ボックスでのテキスト選択中の標準コピー挙動ガード（`Ctrl+C` / `⌘C`）のプラットフォーム追従
- フッターヒント（`shortcutHints`）およびアクションメニュー（`shortcutHint`）の表記を OS に応じて動的切り替え（`⌘` ↔ `Ctrl`）
- ヘルプオーバーレイ（`#help-overlay`）内の `<kbd>` バッジを初期化時に OS に応じた表記に動的更新

## Capabilities

### New Capabilities

### Modified Capabilities
- `credential-copy-actions`: macOS では `⌘C`/`⌘⇧C`/`⌥⌘C`、Linux では `Ctrl+C`/`Ctrl+Shift+C`/`Alt+Ctrl+C` によるコピー操作および非干渉ガードの要件を定義。
- `manual-lock`: 検索画面での手動ロックキーを macOS では `⌘L`、Linux では `Ctrl+L` として定義。
- `quickaccess-help-escape`: ヘルプ開閉ショートカットを macOS では `⌘/`、Linux では `Ctrl+/` として定義。

## Impact

- **バックエンド**: `app/src-tauri/src/commands.rs`, `app/src-tauri/src/main.rs` に `get_platform` コマンドを追加
- **フロントエンド**: `app/dist/app.js`, `app/dist/i18n.js`, `app/dist/index.html`
- **対象環境**: Linux 環境で一般的な `Ctrl` ベースのキーバインドおよび表記が利用可能になり、macOS では既存の `⌘` 体験がそのまま維持される

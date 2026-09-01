## 1. バックエンド プラットフォーム取得コマンドの実装

- [x] 1.1 `app/src-tauri/src/commands.rs` に `#[tauri::command] pub fn get_platform() -> &'static str` を追加し、`target_os = "macos"` 時は `"macos"`、それ以外は `"linux"` を返すようにする
- [x] 1.2 `app/src-tauri/src/main.rs` の `invoke_handler!` に `commands::get_platform` を登録する

## 2. フロントエンド キーイベント判定の改修

- [x] 2.1 `app/dist/app.js` で初期化時に `get_platform` を呼び出してプラットフォームを保持する
- [x] 2.2 `app/dist/app.js` に `isPrimaryMod(event)` ヘルパーを追加し、`isHelpToggleShortcut`, 手動ロック（`KeyL`）、`handleActionShortcut`（`KeyC`）のモディファイアキー判定を `isPrimaryMod` を使用するように更新する（テキスト選択がある場合のコピー非干渉ガードは `handleActionShortcut` 側の判定が `isPrimaryMod` に統一されたことで維持される。`hasTextSelectionInSearchBox` 自体はモディファイアキーを見ないため変更不要）

## 3. ショートカット表記の動的切り替え

- [x] 3.1 `app/dist/app.js` の `buildActionsForItem` で、プラットフォームに応じて `shortcutHint`（`⌘C` ↔ `Ctrl+C`、`⌘⇧C` ↔ `Ctrl+Shift+C`、`⌥⌘C` ↔ `Alt+Ctrl+C`）を生成するように更新する
- [x] 3.2 `app/dist/app.js` で、フッターヒント（`shortcutHints`）の表記をプラットフォームに応じて切り替える（`formatShortcutForPlatform` によるJS側テキスト置換方式を採用。`i18n.js` のメッセージ定義自体は変更不要）
- [x] 3.3 `app/dist/app.js` および `index.html` で、ヘルプオーバーレイ（`#help-overlay`）内の `<kbd>` バッジの表記をプラットフォームに応じて動的に置換する（`data-mod-kbd` 属性を付与した5要素のみ対象。グローバルホットキー行の `⇧⌘Space` はIssue #147のスコープのため対象外。あわせて既存の表記順不整合（`⇧⌘C` → `⌘⇧C`）を `shortcutHints`/spec.md と揃うよう修正）

## 4. ビルド & 動作検証

- [x] 4.1 `cargo check` および `cargo test` を実行し、バックエンドテストがすべて通ることを確認する
- [ ] 4.2 Linux 環境で `cargo run` を実行し、`Ctrl+C`、`Ctrl+Shift+C`、`Alt+Ctrl+C`、`Ctrl+L`、`Ctrl+/` が期待通り動作すること、およびフッター・メニュー・ヘルプの表記が `Ctrl` ベースになっていることを確認する

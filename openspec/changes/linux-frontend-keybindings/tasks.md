## 1. バックエンド プラットフォーム取得コマンドの実装

- [ ] 1.1 `app/src-tauri/src/commands.rs` に `#[tauri::command] pub fn get_platform() -> &'static str` を追加し、`target_os = "macos"` 時は `"macos"`、それ以外は `"linux"` を返すようにする
- [ ] 1.2 `app/src-tauri/src/main.rs` の `invoke_handler!` に `commands::get_platform` を登録する

## 2. フロントエンド キーイベント判定の改修

- [ ] 2.1 `app/dist/app.js` で初期化時に `get_platform` を呼び出してプラットフォームを保持する
- [ ] 2.2 `app/dist/app.js` に `isPrimaryMod(event)` ヘルパーを追加し、`isHelpToggleShortcut`, 手動ロック（`KeyL`）、`handleActionShortcut`（`KeyC`）のモディファイアキー判定を `isPrimaryMod` を使用するように更新する（テキスト選択がある場合のコピー非干渉ガードが維持されることを確認）

## 3. ショートカット表記の動的切り替え

- [ ] 3.1 `app/dist/app.js` の `buildActionsForItem` で、プラットフォームに応じて `shortcutHint`（`⌘C` ↔ `Ctrl+C`、`⌘⇧C` ↔ `Ctrl+Shift+C`、`⌥⌘C` ↔ `Alt+Ctrl+C`）を生成するように更新する
- [ ] 3.2 `app/dist/i18n.js` および `app.js` で、フッターヒント（`shortcutHints`）の表記をプラットフォームに応じて切り替える
- [ ] 3.3 `app/dist/app.js` または `i18n.js` で、ヘルプオーバーレイ（`#help-overlay`）内の `<kbd>` バッジの表記をプラットフォームに応じて動的に置換する

## 4. ビルド & 動作検証

- [ ] 4.1 `cargo check` および `cargo test` を実行し、バックエンドテストがすべて通ることを確認する
- [ ] 4.2 Linux 環境で `cargo run` を実行し、`Ctrl+C`、`Ctrl+Shift+C`、`Alt+Ctrl+C`、`Ctrl+L`、`Ctrl+/` が期待通り動作すること、およびフッター・メニュー・ヘルプの表記が `Ctrl` ベースになっていることを確認する

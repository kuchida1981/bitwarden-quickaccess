## 1. 実装

- [ ] 1.1 `app/src-tauri/src/i18n.rs` の `Messages` 構造体に `pub open_quickaccess_label: &'static str` を追加し、`JA`(例: `"クイックアクセスを開く (⇧⌘Space)"`)/`EN`(例: `"Open Quick Access (⇧⌘Space)"`)にラベルを追加する。
- [ ] 1.2 `app/src-tauri/src/tray.rs` に `const OPEN_QUICKACCESS_ITEM_ID: &str = "open_quickaccess";` を追加する。
- [ ] 1.3 `setup_tray` 内で `open_quickaccess_item`(`MenuItem::with_id`、ラベルは `m.open_quickaccess_label`)を作成し、`Menu::with_items` の項目リストで `hotkey_item` の直後・最初の区切り線の前に挿入する。
- [ ] 1.4 `on_menu_event` の `match` に `OPEN_QUICKACCESS_ITEM_ID => crate::popup::toggle_popup(app),` を追加する。

## 2. 動作確認

- [ ] 2.1 `cargo test` を実行し、既存テストが通ることを確認する。
- [ ] 2.2 `cargo clippy --all-targets -- -D warnings` を実行し、警告が無いことを確認する。
- [ ] 2.3 (ユーザー作業)実機で、トレイメニューに新しい項目が表示され、選択するとクイックアクセスが開く/閉じることを確認する。
- [ ] 2.4 (ユーザー作業)実機で、日本語/英語それぞれのロケールでラベルが正しく表示されることを確認する。

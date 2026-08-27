## 1. 実装

- [x] 1.1 `app/src-tauri/src/i18n.rs` の `Messages` 構造体に `pub open_quickaccess_label: &'static str` を追加し、`JA`(例: `"クイックアクセスを開く (⇧⌘Space)"`)/`EN`(例: `"Open Quick Access (⇧⌘Space)"`)にラベルを追加する。(agy, commit 98c5088)
- [x] 1.2 `app/src-tauri/src/tray.rs` に `const OPEN_QUICKACCESS_ITEM_ID: &str = "open_quickaccess";` を追加する。(agy, commit 98c5088)
- [x] 1.3 `setup_tray` 内で `open_quickaccess_item`(`MenuItem::with_id`、ラベルは `m.open_quickaccess_label`)を作成し、`Menu::with_items` の項目リストで `hotkey_item` の直後・最初の区切り線の前に挿入する。(agy, commit 98c5088)
- [x] 1.4 `on_menu_event` の `match` に `OPEN_QUICKACCESS_ITEM_ID => crate::popup::toggle_popup(app),` を追加する。(agy, commit 98c5088)

## 2. 動作確認

- [x] 2.1 `cargo test` を実行し、既存テストが通ることを確認する。(2026-08-27 全12テスト成功)
- [x] 2.2 `cargo clippy --all-targets -- -D warnings` を実行し、警告が無いことを確認する。(2026-08-27 警告なし)
- [x] 2.3 (ユーザー作業)実機で、トレイメニューに新しい項目が表示され、選択するとクイックアクセスが開く/閉じることを確認する。(2026-08-27 確認完了、問題なし)

## 3. セキュリティレビュー

- [x] 3.1 `/security-review` を実行する。(2026-08-27 実行、指摘なし。既存の `popup::toggle_popup` を新しいメニュー項目から呼ぶのみで新規の攻撃面なし)

## 4. 実機確認で発覚した重複表示の修正

- [x] 4.1 実機確認(2.3)時に、ホットキー登録成功時の「ホットキー: ⇧⌘Space」表示が新規の「クイックアクセスを開く (⇧⌘Space)」と重複していることが判明(design.md参照)。
- [x] 4.2 `app/src-tauri/src/tray.rs` の `hotkey_item` を、ホットキー登録に失敗した場合(`hotkey_warning: Some`)のみメニューに含めるよう変更(`Menu::with_items` を可変長の `Vec<&dyn IsMenuItem<Wry>>` に変更)。登録成功時は `hotkey_item` 自体をメニューから除外する。(Claude Code実装)
- [x] 4.3 未使用になった `app/src-tauri/src/i18n.rs` の `Messages::hotkey_registered` フィールドを `JA`/`EN` 両方から削除する。(Claude Code実装)
- [x] 4.4 `cargo test` / `cargo clippy --all-targets -- -D warnings` を再実行し、問題ないことを確認する。(2026-08-27 全12テスト成功・警告なし)

## 5. コードレビュー

- [x] 5.1 `/code-review` を実行する(重複表示修正を含む最終的な diff を対象とする)。(2026-08-27 実行、指摘なし)

## 6. 動作確認(2回目、重複表示修正後)

- [ ] 6.1 (ユーザー作業)実機で、ホットキー登録成功時にトレイメニューへ「ホットキー: ⇧⌘Space」の重複表示が出ないことを確認する。
- [ ] 6.2 (ユーザー作業)実機で、日本語/英語それぞれのロケールで「クイックアクセスを開く」ラベルが正しく表示されることを確認する。

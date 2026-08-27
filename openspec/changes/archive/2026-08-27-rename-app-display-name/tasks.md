## 1. フロントエンド(HTML/JS)実装

- [x] 1.1 `app/dist/i18n.js` の `MESSAGES.ja` / `MESSAGES.en` に新規キー `appDisplayName` を追加する(`ja: "Bitwarden クイックアクセス"`, `en: "Bitwarden Quick Access"`)。(agy, commit c332ac1)
- [x] 1.2 `app/dist/index.html` の `<title>bw-quickaccess</title>` を `<title data-i18n="appDisplayName">Bitwarden クイックアクセス</title>` に変更する。(agy, commit c332ac1)
- [x] 1.3 `app/dist/index.html` のアンロック画面 `<h1>bw-quickaccess</h1>` を `<h1 data-i18n="appDisplayName">Bitwarden クイックアクセス</h1>` に変更する。(agy, commit c332ac1)

## 2. バックエンド(Rust)実装

- [x] 2.1 `app/src-tauri/src/i18n.rs` の `Messages` 構造体に `pub app_display_name: &'static str` を追加し、`JA`(`"Bitwarden クイックアクセス"`)/`EN`(`"Bitwarden Quick Access"`)に値を追加する。(agy, commit c332ac1)
- [x] 2.2 `app/src-tauri/src/popup.rs` の `create_popup_window` 内で `app.state::<crate::i18n::Lang>()` から言語を取得し、`.title("bw-quickaccess")` を `.title(crate::i18n::messages(lang).app_display_name)` に変更する。(agy, commit c332ac1)

## 3. 動作確認

- [x] 3.1 `cargo test` を実行し、既存テストが通ることを確認する。(2026-08-27 全38テスト成功)
- [x] 3.2 `cargo clippy --all-targets -- -D warnings` を実行し、警告が無いことを確認する。(2026-08-27 警告なし)
- [x] 3.3 `node --check app/dist/app.js` および `node --check app/dist/i18n.js` で構文エラーが無いことを確認する。(2026-08-27 確認済み)
## 4. セキュリティレビュー・コードレビュー

- [x] 4.1 `/security-review` を実行する。(2026-08-27 実行、指摘なし。全て開発者定義の定数文字列の置き換えのみで、外部入力・新規IPC経路なし)
- [x] 4.2 `/code-review` を実行する。(2026-08-27 実行、指摘なし。`Lang` state取得順序、`textContent`によるtitle更新の安全性を確認済み)

## 5. 実機確認

- [x] 5.1 (ユーザー作業)実機で、日本語/英語それぞれのロケールで、アンロック画面の見出しが新しい表示名になっていることを確認する。(2026-08-27 確認完了、問題なし)
- [x] 5.2 (ユーザー作業)実機で、Finder上のアプリ名(`bw-quickaccess`)やトレイメニューのAbout表示が変わっていないことを確認する(回帰確認)。(2026-08-27 確認完了、問題なし)

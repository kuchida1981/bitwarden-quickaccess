## 1. 実装

- [x] 1.1 `app/dist/index.html` の `#search-box` input要素に `spellcheck="false"`、`autocorrect="off"`、`autocapitalize="off"` を追加する(既存の `autocomplete="off"` は維持)

## 2. 動作確認

- [x] 2.1 `cargo tauri build`(または `cargo run`)でアプリを起動し、検索ボックスに文字を入力してもmacOSのインラインサジェスト/スペルチェック/オートコレクトのポップアップが表示されないことを目視確認する
- [x] 2.2 検索ボックスに入力した直後に↑/↓キーで検索結果一覧の行フォーカスが問題なく移動することを確認する

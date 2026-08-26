## 1. Escapeキーでポップアップを閉じる(#54)

- [x] 1.1 `app/dist/app.js` の `searchBox` の `keydown` ハンドラの先頭(`actionMenuOpen` の分岐の直後)に、`event.key === "Escape"` の場合に `event.preventDefault()` の上で `invoke("hide_popup").catch(() => {})` を呼ぶ分岐を追加する(design.md 決定3参照。新規Rustコマンドは不要)
- [x] 1.2 動作確認: 検索ポップアップ表示中(ヘルプ・アクションメニューいずれも非表示)に `Esc` を押すとポップアップが閉じることを確認する(実機確認が必要)

## 2. ヘルプオーバーレイのマークアップとスタイル

- [x] 2.1 `app/dist/index.html` の `#search-screen` 内、`#feedback` の後に `<div id="help-overlay">` を追加する。中身は design.md 決定5の表に基づき、キーとその説明を並べた静的なリスト(例: `<dl>` 要素、各説明に `data-i18n` 属性)にする。`⇧⌘Space`・`↑`/`↓`・`→`・`←`・`⌘C`・`⇧⌘C`・`⌥⌘C`・`Enter`・`Esc`・`⌘/` の10項目すべてを含める
- [x] 2.2 `app/dist/i18n.js` に、2.1で使う `data-i18n` キー(例: `helpTitle`, `helpTogglePopup`, `helpMoveFocus`, `helpOpenMenu`, `helpCloseMenu`, `helpCopyUsername`, `helpCopyPassword`, `helpCopyTotp`, `helpOpenBrowser`, `helpClose`, `helpToggleHelp`)を日本語・英語両方の辞書に追加する
- [x] 2.3 `app/dist/style.css` に `#search-screen` を `position: relative` にし、`#help-overlay` を `position: absolute; inset: 0` で覆うオーバーレイスタイル(既定 `display: none`、`.visible` クラスで `display: block`、背景色・パディング・スクロール可能)を追加する。キーと説明を横並びで見やすく表示するスタイルも加える

## 3. ヘルプの開閉ロジックとキー入力優先順位の整理(#53)

- [x] 3.1 `app/dist/app.js` の状態変数に `let helpOpen = false;` を追加する。DOM参照に `const helpOverlay = document.getElementById("help-overlay");` を追加する
- [x] 3.2 `openHelp()` / `closeHelp()` 関数を新設する。`openHelp()` は `actionMenuOpen` が `true` なら先に `closeActionMenu()` を呼び、その後 `helpOpen = true` にして `helpOverlay.classList.add("visible")` する。`closeHelp()` は `helpOpen = false` にして `helpOverlay.classList.remove("visible")` する
- [x] 3.3 `handleHelpKeydown(event)` 関数を新設する: `Escape` または(`event.metaKey && event.code === "Slash" && !event.shiftKey && !event.altKey`)であれば `event.preventDefault()` の上で `closeHelp()` を呼ぶ。それ以外のキーはすべて `event.preventDefault()` して無視する
- [x] 3.4 `searchBox` の `keydown` ハンドラの優先順位を design.md 決定3の通りに書き換える: `helpOpen` を最優先でチェックして `handleHelpKeydown` に委譲し、その次に既存の `actionMenuOpen` チェック、その次に(1.1で追加した)`Escape`、その次に `⌘/`(`event.metaKey && event.code === "Slash" && !event.shiftKey && !event.altKey` で `openHelp()` を呼ぶ)、最後に既存の `ArrowDown`/`ArrowUp`/`ArrowRight`/`handleActionShortcut` の順にする
- [x] 3.5 `handleShown()` に `helpOpen = false; helpOverlay.classList.remove("visible");` を追加し、ポップアップの再表示時にヘルプが開いたまま残らないようにする(`quickaccess-action-menu` で `actionMenuOpen` に対して行ったのと同じ対応)

## 4. 動作確認・仕上げ

- [x] 4.1 `node --check app/dist/app.js` が通ることを確認する
- [x] 4.2 検索ポップアップ表示中に `⌘/` を押し、ショートカット一覧が表示されることを確認する(実機確認が必要)。ユーザー確認済み
- [x] 4.3 ヘルプ表示中に `⌘/` を押すと閉じて検索画面に戻ることを確認する(実機確認が必要)。ユーザー確認済み
- [x] 4.4 ヘルプ表示中に `Esc` を押すとヘルプが閉じるが、ポップアップ自体は閉じたままにならない(表示され続ける)ことを確認する(実機確認が必要)。ユーザー確認済み
- [x] 4.5 ヘルプ表示中に文字を入力しても検索ボックスの内容が変化しないことを確認する(実機確認が必要)。ユーザー確認済み
- [ ] 4.6 アクションメニュー表示中に `⌘/` を押すと、メニューが閉じてヘルプが表示されることを確認する(実機確認が必要)。当初ユーザー確認済みとしていたが、`/code-review` で `actionMenuOpen` の分岐が `⌘/` の判定より先に評価され実際には機能していないバグが発覚(design.md 決定4違反)。修正済みだが、修正後の再確認はまだ行っていない
- [x] 4.7 アクションメニュー表示中に `Esc` を押すと、従来通りメニューだけが閉じ、ポップアップは閉じないことを確認する(デグレ確認、実機確認が必要)。ユーザー確認済み
- [x] 4.8 `specs/quickaccess-help-escape/spec.md` の各シナリオが満たされていることを確認する

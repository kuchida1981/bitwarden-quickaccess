## 1. セキュリティ修正: 検索結果DTOの導入(バックエンド)

- [ ] 1.1 `app/src-tauri/src/commands.rs` に `SearchResultItem` 構造体(`id`, `name`, `username: Option<String>`, `has_password: bool`, `has_totp: bool`, `has_url: bool`)と `impl From<VaultItemSummary> for SearchResultItem` を追加する(design.md 決定1のコードをそのまま実装する)
- [ ] 1.2 `search_items` コマンドの戻り値型を `Result<Vec<SearchResultItem>, String>` に変更し、`client.search_items(&query).await?` の結果を `.into_iter().map(SearchResultItem::from).collect()` してから返すようにする
- [ ] 1.3 単体テスト: `VaultItemSummary`(password/totp/uriあり・なしの組み合わせ)から `SearchResultItem` への変換が正しい真偽値を返すことを検証するテストを追加する(`app/src-tauri/src/commands.rs` の既存 `#[cfg(test)] mod tests` に追加)
- [ ] 1.4 `cd app/src-tauri && cargo build && cargo clippy --all-targets -- -D warnings && cargo test` が通ることを確認する

## 2. フロントエンド: 新しいレスポンス形状への追従

- [ ] 2.1 `app/dist/app.js` の `renderResults()` 内、`item.login && item.login.username` を `item.username` に修正する(`SearchResultItem` はもう `login` オブジェクトを持たずフラットな形になるため)
- [ ] 2.2 動作確認: 検索してユーザー名を持つアイテムが一覧に正しく表示されることを確認する(実機確認が必要)

## 3. アクションメニュー: 構築ロジックとキーボード操作(フロントエンド)

- [ ] 3.1 `app/dist/app.js` に、`SearchResultItem` を受け取り実行可能なアクションの配列を返す関数 `buildActionsForItem(item)` を新設する(design.md 決定4のコード例を参照。`enabled` が `false` の項目は配列に含めない)。各アクション項目は `{ key, labelKey, shortcutHint }` の形にする(`labelKey` は将来のi18n辞書キー参照用。i18n辞書(`app/dist/i18n.js`)に `actionCopyUsername` / `actionCopyPassword` / `actionCopyTotp` / `actionOpenBrowser` の4キーを追加し、`t(labelKey)` で解決する)
- [ ] 3.2 既存の `handleActionShortcut` 内でコピー/ブラウザ起動を行っている処理(`⌘C` 等のキー判定の中身)を、`key`(`"username"` / `"password"` / `"totp"` / `"browser"`)を受け取ってアクションを実行する共通関数(例: `executeItemAction(item, key)`)に切り出す。`handleActionShortcut` からも、後述のメニュー実行処理からも、この共通関数を呼ぶようにする(実装の重複を避ける)
- [ ] 3.3 `app.js` の状態変数に `let actionMenuOpen = false;` と `let actionMenuFocusIndex = -1;` を追加する
- [ ] 3.4 `searchBox` の `keydown` ハンドラを拡張し、`actionMenuOpen` の値に応じて以下のように分岐する(design.md 決定3参照):
  - `actionMenuOpen === false` の状態で `ArrowRight`: フォーカス中アイテムに対して `buildActionsForItem` を呼び、結果が1件以上あれば `actionMenuOpen = true`, `actionMenuFocusIndex = 0` にして再描画する
  - `actionMenuOpen === true` の状態で `ArrowDown` / `ArrowUp`: `actionMenuFocusIndex` をメニュー項目数の範囲内で移動して再描画する
  - `actionMenuOpen === true` の状態で `Enter`: `actionMenuFocusIndex` が指すアクションを 3.2 の共通関数で実行する
  - `actionMenuOpen === true` の状態で `ArrowLeft` または `Escape`: `actionMenuOpen = false` にして再描画する(ポップアップ全体を閉じるわけではない。ポップアップを閉じるEscapeの実装は別issue #54)
  - `actionMenuOpen === true` の間は、既存の `ArrowDown`/`ArrowUp`(アイテム間移動)や検索文字入力を処理しない(上記以外のキーは無視する)
  - `handleActionShortcut`(`⌘C` 等のダイレクトショートカット)は `actionMenuOpen` の値によらず常に動作させる(変更しない)

## 4. アクションメニュー: 描画とクリック操作(フロントエンド)

- [ ] 4.1 `renderResults()` を拡張し、`actionMenuOpen === true` かつ対象アイテムがフォーカス中の行である場合、既存の `.hints` の代わりに `<ul class="action-menu">` を描画する。各 `<li>` にはラベルとショートカットヒントを表示し、`actionMenuFocusIndex` に一致する項目には `focused` 相当のクラスを付与する
- [ ] 4.2 メニュー項目の `<li>` に `click` イベントハンドラを追加し、クリックされたら 3.2 の共通関数でそのアクションを実行する
- [ ] 4.3 `app/dist/style.css` に `.action-menu` とその `li` / `li.focused` のスタイルを追加する(既存の `#results li .hints` の見た目を踏襲する簡素なスタイルでよい)

## 5. 動作確認・仕上げ

- [ ] 5.1 `cd app/src-tauri && cargo build && cargo clippy --all-targets -- -D warnings && cargo test` および `node --check app/dist/app.js` が通ることを確認する
- [ ] 5.2 全フィールド(ユーザー名・パスワード・TOTP・URL)を持つアイテムでRightキーを押し、4項目すべてが表示されることを確認する(実機確認が必要)
- [ ] 5.3 パスワードやTOTPを持たないアイテムでRightキーを押し、該当項目が表示されないことを確認する(実機確認が必要)
- [ ] 5.4 メニュー展開中に↑/↓でフォーカス移動、Enterで実行、クリックでも実行できることを確認する(実機確認が必要)
- [ ] 5.5 メニュー展開中にLeftキーで通常の一覧操作に戻れることを確認する(実機確認が必要)
- [ ] 5.6 メニューを開いていない状態でも、従来通り `⌘C` 等のダイレクトショートカットが動作することを確認する(実機確認が必要、既存機能のデグレがないことの確認)
- [ ] 5.7 `specs/quickaccess-action-menu/spec.md` および `specs/incremental-item-search/spec.md`(ADDED分)の各シナリオが満たされていることを確認する

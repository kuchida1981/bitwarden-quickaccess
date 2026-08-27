## 1. 実現可否調査(完了)

- [x] 1.1 (ユーザー作業)実機の `bw serve` (`/list/object/items`) レスポンスを確認。アイコン関連フィールドは存在せず、ドメイン抽出+外部アイコン取得(パスB)が必要と判明。
- [x] 1.2 `design.md` にパスB確定の決定事項を反映。

## 2. バックエンド実装: ドメイン抽出

- [x] 2.1 `app/src-tauri/src/backend/http_client.rs` に `LoginDetail::icon_domain()` を実装(agy, commit 8234aa1)。
- [x] 2.2 `app/src-tauri/src/commands.rs` の `SearchResultItem` に `icon_domain: Option<String>` を追加し、`From<VaultItemSummary>` に反映(agy, commit 8234aa1)。
- [x] 2.3 `icon_domain` のテストを追加(agy, commit 8234aa1)。

## 3. フロントエンド実装: アイコン取得・表示

- [x] 3.1 実装前に、想定エンドポイント(`https://icons.bitwarden.net/{domain}/icon.png` 形式)が実際に画像を返すか確認する。(2026-08-27 `curl` で `https://icons.bitwarden.net/amazon.co.jp/icon.png` を確認し、`200 image/png` を確認済み)
- [x] 3.2 `app/dist/app.js` の `renderResults` に `icon_domain` を持つアイテムのアイコン表示を追加(agy, commit be7e24f)。
- [x] 3.3 `<img>` の `onerror` によるプレースホルダーへのフォールバックを実装(agy, commit be7e24f)。
- [x] 3.4 アイコン読み込みは非同期(`<img>` 標準の読み込みのみ、同期待ちなし)であることを確認(agy, commit be7e24f)。
- [x] 3.5 `app/dist/style.css` にアイコン・プレースホルダーのスタイルを追加(agy, commit be7e24f)。

## 4. セキュリティレビュー

- [x] 4.1 `/security-review` を実行し、外部アイコンサービスへの通信追加によるセキュリティ・プライバシー上の懸念がないか確認する。指摘があれば対応する。(2026-08-27 実行、confidence 8/10以上の指摘なし。ドメイン抽出の簡易パース(userinfo等のリーク可能性)と、外部サービスへの常時通信自体は design.md で既知・許容済みのトレードオフとして検討済み)

## 5. コードレビュー指摘対応

- [x] 5.1 `/code-review` を実行(2026-08-27)。`icon_domain()` の手動文字列分割がuserinfo(認証情報)漏洩・ポート番号混入(セルフホストVaultwardenで実在ドメイン扱いされずアイコン取得が常に失敗)を引き起こす指摘を受領。
- [x] 5.2 `icon_domain()` を `reqwest::Url::parse().host_str()` による正式なURLパースに置き換え、userinfo・ポート付きURIのテストケースを追加(agy, commit cee6625)。

## 6. 動作確認(1回目)

- [x] 6.1 `cargo test` を実行し、既存テストおよび追加したテストが通ることを確認する。(2026-08-27 全12テスト成功、修正後再実行済み)
- [x] 6.2 `cargo clippy --all-targets -- -D warnings` を実行し、警告が無いことを確認する。(2026-08-27 警告なし、修正後再実行済み)
- [x] 6.3 (ユーザー作業)実機で、URIを持つアイテム・持たないアイテムそれぞれについて、一覧行にアイコン(またはプレースホルダー)が表示されることを確認する。(2026-08-27 確認完了、問題なし)

## 7. 実機確認で発覚した不具合の修正(スコープ内対応)

- [x] 7.1 実機確認(6.3)時に、アイコンのチラつきと、矢印キーでの行フォーカス移動がマウスカーソル位置の行へ巻き戻る不具合を発見。原因は `renderResults()` がフォーカス変更のたびに一覧全体(`<li>`とその子要素)を作り直しており、(a) `<img>` が毎回再生成されチラつく、(b) カーソル直下に出現した新しい`<li>`に対しブラウザが`mouseenter`を誤発火させ`focusedIndex`を上書きする、という2つの副作用を引き起こしていたため(design.md 参照)。ユーザーの了承を得て本changeのスコープ内で修正する。
- [x] 7.2 `app/dist/app.js` の `renderResults()` から末尾ブロック(ヒント/アクションメニュー)生成を `buildTrailingBlock()` に分離し、フォーカス変更専用の `updateFocusRows()`(矢印キー・マウスホバー用、影響を受ける2行のみ更新)と `refreshFocusedRowTrailing()`(アクションメニュー開閉・メニュー内移動用、フォーカス行1行のみ更新)を追加。`moveFocus` / 行の `mouseenter` / `openActionMenu` / `closeActionMenu` / `handleActionMenuKeydown` の呼び出し元を、一覧全体を再構築する `renderResults()` からこれらの部分更新関数に切り替えた(Claude Code実装、`currentItems` 自体が変わる検索結果更新時のみ従来通り `renderResults()` を使用)。
- [x] 7.3 `node --check app/dist/app.js` で構文エラーが無いことを確認。全呼び出し箇所(`moveFocus`, `mouseenter`, `openActionMenu`, `closeActionMenu`, `handleActionMenuKeydown`)が新しい部分更新関数に統一されていることをコード確認。

## 8. 動作確認(2回目、不具合修正後)

- [x] 8.1 (ユーザー作業)実機で、矢印キーによる行フォーカス移動時にアイコンがチラつかないことを確認する。(2026-08-27 確認完了、問題なし)
- [x] 8.2 (ユーザー作業)実機で、マウスカーソルを一覧上に静止させたまま矢印キーで行フォーカスを移動しても、カーソル位置の行へ巻き戻らないことを確認する。(2026-08-27 確認完了、問題なし)
- [x] 8.3 (ユーザー作業)実機で、アクションメニューの開閉・メニュー内の矢印キー移動が従来通り正しく動作することを確認する(回帰確認)。(2026-08-27 確認完了、問題なし)
- [x] 8.4 (ユーザー作業)実機で、オフライン状態(またはアイコン取得先に到達できない状態)でもクイックアクセスの検索・行フォーカス移動が問題なく行え、プレースホルダーが表示されることを確認する。(2026-08-27 確認完了、問題なし)

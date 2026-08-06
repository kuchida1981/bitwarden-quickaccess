## 1. フィールド選択画面の表示順序変更

- [ ] 1.1 `lib/fields.sh` の `bwqa_build_field_rows()` の jq 配列順序をユーザー名→パスワード→TOTP に変更し、周辺コメント(21行目「password を先頭行にして...」)も新しい順序の意図に合わせて更新する
- [ ] 1.2 `test/lib/fields.bats` に、全フィールドが揃っている場合の表示順序、および一部フィールドが欠けている場合の表示順序(欠けたフィールドが除外されつつ順序が保たれること)を確認するテストを追加する

## 2. bw CLI呼び出し中のローディングメッセージ

- [ ] 2.1 `lib/session.sh` `bwqa_unlock()` の `bw unlock` 実行直前に `bwqa_log` でロック解除中であることを示すメッセージを出す
- [ ] 2.2 `lib/search.sh` `bwqa_fetch_items()` の `bw list items`(`bwqa_bw list items`)実行直前に `bwqa_log` で vault 読み込み中であることを示すメッセージを出す
- [ ] 2.3 `lib/fields.sh` `bwqa_get_item_summary()` の `bw get item` 実行直前に `bwqa_log` でアイテム情報取得中であることを示すメッセージを出す
- [ ] 2.4 `lib/fields.sh` `bwqa_copy_field_internal()` の `bw get username/password/totp` 実行直前に `bwqa_log` で値取得中であることを示すメッセージを出す(メッセージにアイテム名やフィールド値そのものを含めないこと)
- [ ] 2.5 `test/lib/session.bats` / `test/lib/search.bats` / `test/lib/fields.bats` に、各メッセージが stderr に出力されること、かつ機密情報(アイテム名・値)を含まないことを確認するテストを追加する

## 3. ドキュメント・仕上げ

- [ ] 3.1 README.md にフィールド選択画面の表示順序やローディングメッセージに関する記載があれば更新する(なければスキップ)
- [ ] 3.2 `test/lib/*.bats` 全体とシェルスクリプト全体(`shellcheck`)がパスすることを確認する

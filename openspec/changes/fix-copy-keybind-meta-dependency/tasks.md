## 1. 検索画面(lib/search.sh)のキーバインド変更

- [x] 1.1 `--header` の文言を `alt-u: ユーザー名  alt-p: パスワード` から `ctrl-o: ユーザー名  ctrl-r: パスワード` に変更する
- [x] 1.2 `--bind="alt-u:..."` を `--bind="ctrl-o:..."` に変更する
- [x] 1.3 `--bind="alt-p:..."` を `--bind="ctrl-r:..."` に変更する
- [x] 1.4 ファイル冒頭のコメント(`alt-u/p、ctrl-t バインドでは...` 等、`lib/fields.sh` 側にある参照コメントも含む)を新キー名に更新する

## 2. フィールド選択画面(lib/fields.sh)のキーバインド変更

- [x] 2.1 `bwqa_build_field_rows` のラベル文字列 `ユーザー名をコピー (alt-u)` / `パスワードをコピー (alt-p)` を `(ctrl-o)` / `(ctrl-r)` に変更する
- [x] 2.2 `--header` の文言を `alt-p: password  alt-u: username` から `ctrl-r: password  ctrl-o: username` に変更する
- [x] 2.3 `--bind="alt-p:..."` を `--bind="ctrl-r:..."` に変更する
- [x] 2.4 `--bind="alt-u:..."` を `--bind="ctrl-o:..."` に変更する

## 3. テスト更新

- [x] 3.1 `test/lib/fields.bats` の `ユーザー名をコピー (alt-u)` を期待する assertion を `(ctrl-o)` に更新する
- [x] 3.2 `bats test/lib/*.bats` を実行し全件 pass することを確認する

## 4. ドキュメント更新

- [x] 4.1 README.md のキーバインド説明(検索画面・フィールド選択画面の両方の箇条書き)を `ctrl-o`/`ctrl-r` に更新する
- [x] 4.2 README.md の「`alt-p`/`alt-u` はターミナルエミュレータの Meta キー送信設定に依存します」という注記を削除する

## 5. 動作確認

- [x] 5.1 shellcheck (`shellcheck -x bin/bw-quickaccess`) が通ることを確認する
- [ ] 5.2 macOS 実機(Alacritty 等、Meta キー設定を有効化していない環境)で `ctrl-o`/`ctrl-r`/`ctrl-t` による検索画面からの直接コピーが機能することを確認する
- [ ] 5.3 フィールド選択画面でも同様に `ctrl-o`/`ctrl-r`/`ctrl-t` が機能し、`ctrl-p`/`ctrl-n`(選択移動)・`ctrl-u`(クエリ編集)が fzf のネイティブ動作のまま維持されていることを確認する
- [ ] 5.4 対象フィールドが存在しないアイテムでキーバインド押下時のフィードバック表示を確認する

## 6. OpenSpec 同期

- [ ] 6.1 `/opsx:archive` で本 change をアーカイブし、`vault-item-search` / `credential-clipboard-copy` の delta spec を main spec に同期する

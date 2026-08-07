## 1. lib/search.sh の更新

- [x] 1.1 `--bind="ctrl-p:..."` / `--bind="ctrl-u:..."` を `alt-p` / `alt-u` に変更する
- [x] 1.2 `--header` の文言(`ctrl-u: ユーザー名  ctrl-p: パスワード`)を `alt-u` / `alt-p` に更新する

## 2. lib/fields.sh の更新

- [x] 2.1 `--bind="ctrl-p:..."` / `--bind="ctrl-u:..."` を `alt-p` / `alt-u` に変更する
- [x] 2.2 `--header` の文言(`ctrl-p: password  ctrl-u: username`)を `alt-p` / `alt-u` に更新する
- [x] 2.3 `bwqa_build_field_rows` の行ラベル表記(`パスワードをコピー (ctrl-p)` / `ユーザー名をコピー (ctrl-u)`)を `(alt-p)` / `(alt-u)` に更新する

## 3. ドキュメント更新

- [x] 3.1 README.md の使い方セクション(検索画面・フィールド選択画面それぞれのキーバインド説明)を `alt-p` / `alt-u` に更新する
- [x] 3.2 README.md に、Alt キーの送信がターミナル設定に依存する場合がある旨(例: macOS 標準 Terminal.app の「Use Option as Meta Key」)を注記する

## 4. 動作確認(手動)

- [x] 4.1 検索画面・フィールド選択画面それぞれで `alt-p`(パスワード)/`alt-u`(ユーザー名)/`ctrl-t`(TOTP)のコピーが機能することを確認する
- [x] 4.2 検索画面・フィールド選択画面それぞれで `ctrl-p`/`ctrl-n`(選択の上下移動)と `ctrl-u`(検索クエリのカーソル左側クリア)が fzf のネイティブ動作に戻っていることを確認する
- [x] 4.3 手元のターミナル(またはチームで使われている代表的なターミナル)で `alt-p`/`alt-u` が実際に機能するか確認し、機能しない場合は README の注記内容を実態に合わせて調整する

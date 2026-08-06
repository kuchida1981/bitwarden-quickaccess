## Why

実際に動作確認したところ、リスクの低い UX 上の粗さが2点見つかった。(1) フィールド選択画面の項目順序(パスワード→ユーザー名→TOTP)が実際の利用感覚と合っておらず、Enter キーで意図せずパスワードが優先コピーされやすい。(2) `bw` CLI 呼び出し(session unlock、vault一覧取得、アイテム詳細取得、フィールド値取得)の間は何も表示されず、ツールがフリーズしているように見える。どちらも独立した小さな改善のため、まとめて対応する。

## What Changes

- フィールド選択画面の表示順序を「パスワード→ユーザー名→TOTP」から「ユーザー名→パスワード→TOTP」に変更する(`lib/fields.sh` の `bwqa_build_field_rows()`)
- 上記の表示順序をコード上の実装詳細ではなく、spec 上の明示的な要件として固定する
- 以下の `bw` CLI 呼び出しの直前に、`bwqa_log` 経由で処理中であることを示すメッセージを stderr に出力する
  - `lib/session.sh` `bwqa_unlock()`(`bw unlock`)
  - `lib/search.sh` `bwqa_fetch_items()`(`bw list items`)
  - `lib/fields.sh` `bwqa_get_item_summary()`(`bw get item`)
  - `lib/fields.sh` `bwqa_copy_field_internal()`(`bw get username/password/totp`)
- 非同期ロード(fzf 起動後の `reload` によるプログレッシブ表示)や fzf 側のスピナー表示は本 change のスコープ外とする。まずは呼び出し前のメッセージ出力のみ対応する

## Capabilities

### New Capabilities
- `loading-feedback`: `bw` CLI 呼び出し中(session unlock、vault一覧取得、アイテム詳細取得、フィールド値取得)に、処理中であることを示すメッセージを表示する

### Modified Capabilities
- `credential-clipboard-copy`: フィールド選択画面の表示順序を「ユーザー名→パスワード→TOTP」に固定する要件を追加する(現状は順序が spec 上未規定で、実装(`lib/fields.sh`)がパスワード優先の順序になっていた)

## Impact

- `lib/fields.sh`: `bwqa_build_field_rows()` の順序変更、`bwqa_get_item_summary()` / `bwqa_copy_field_internal()` へのローディングメッセージ追加
- `lib/search.sh`: `bwqa_fetch_items()` へのローディングメッセージ追加
- `lib/session.sh`: `bwqa_unlock()` へのローディングメッセージ追加
- `test/lib/fields.bats`, `test/lib/search.bats`, `test/lib/session.bats`: 上記変更に対応するテストの追加・更新
- 破壊的変更なし。既存のキーバインド(`ctrl-p`/`ctrl-u`/`ctrl-t`)や外部インターフェースへの変更はない

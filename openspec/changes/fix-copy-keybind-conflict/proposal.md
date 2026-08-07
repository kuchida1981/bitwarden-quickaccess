## Why

検索画面・フィールド選択画面の `ctrl-p`(パスワードコピー)と `ctrl-u`(ユーザー名コピー)は、fzf がデフォルトで `ctrl-p` を `up-match`(選択を1つ上へ)、`ctrl-u` を `unix-line-discard`(検索クエリのカーソル左側をクリア)に割り当てているキーと衝突している。`--bind` による再バインドはデフォルト動作を完全に上書きするため、`ctrl-p` を押しても選択が上に動かなくなり(`ctrl-n` による下方向のみ生存)、`ctrl-u` を押してもクエリがクリアされなくなっている。fzf のネイティブな操作性を壊しているため、コピー用ショートカットのキー割り当てを見直す(issue #13)。

## What Changes

- パスワードコピーのキーバインドを `ctrl-p` から `alt-p` に変更する(**BREAKING**: 既存ユーザーが覚えている `ctrl-p` は使えなくなる)
- ユーザー名コピーのキーバインドを `ctrl-u` から `alt-u` に変更する(**BREAKING**: 同上)
- TOTP コピーのキーバインド `ctrl-t` は変更しない(fzf デフォルトと衝突しないため)
- 検索画面・フィールド選択画面の `--header` 表示文言、フィールド選択画面の行ラベル(`(ctrl-p)` 等の表記)を新キーバインドに合わせて更新する
- README.md の使い方セクションのキーバインド説明を更新し、Alt キーがターミナル設定(例: macOS 標準 Terminal.app の「Use Option as Meta Key」)に依存する場合がある旨を注記する

## Capabilities

### New Capabilities

(なし)

### Modified Capabilities

- `credential-clipboard-copy`: 「キーバインドによるショートカットコピー」要件のキー割り当てを `ctrl-u/ctrl-p/ctrl-t` から `alt-u/alt-p/ctrl-t` に変更する
- `copy-feedback`: 「コピー結果のフィードバック表示」要件内でキーバインドを明記している記述を `ctrl-u/ctrl-p/ctrl-t` から `alt-u/alt-p/ctrl-t` に変更する

## Impact

- `lib/search.sh`: `--bind` の `ctrl-p`/`ctrl-u` を `alt-p`/`alt-u` に変更、`--header` 文言を更新
- `lib/fields.sh`: `--bind` の `ctrl-p`/`ctrl-u` を `alt-p`/`alt-u` に変更、`--header` 文言と `bwqa_build_field_rows` のラベル表記を更新
- `README.md`: 使い方セクションのキーバインド説明を更新、Alt キーのターミナル依存に関する注記を追加
- `openspec/specs/credential-clipboard-copy/spec.md`: キーバインドを明記しているシナリオの記述を更新
- `openspec/specs/copy-feedback/spec.md`: キーバインドを明記している要件・シナリオの記述を更新
- `test/lib/fields.bats`: `bwqa_build_field_rows` のラベル表記変更に合わせて期待値を更新

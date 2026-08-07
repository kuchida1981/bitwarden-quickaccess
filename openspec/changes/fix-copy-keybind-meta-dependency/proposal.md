## Why

検索画面・フィールド選択画面のユーザー名/パスワード直接コピーに割り当てている `alt-u`/`alt-p` は、ターミナルが Option キーを Meta キー(ESC プレフィックス)として送信する設定になっていないと動作しない。この設定は Terminal.app・iTerm2・Alacritty などターミナルごとに異なり、デフォルトでは無効な場合が多く、実際に macOS + Alacritty の組み合わせで動作しないことが確認された(Issue #19)。調査の結果、`ctrl-enter`/`shift-enter` は fzf のキー語彙に存在せず、F-Key は macOS のメディアキー割り当てにより同種の環境依存を別の層で抱えることが判明した。ターミナル・OS の設定に依存しない実装に変更する。

## What Changes

- 検索画面・フィールド選択画面のユーザー名コピーのキーバインドを `alt-u` → `ctrl-o` に変更する(**BREAKING**: 既存ユーザーが覚えているキー操作が変わる)
- 検索画面・フィールド選択画面のパスワードコピーのキーバインドを `alt-p` → `ctrl-r` に変更する(**BREAKING**: 同上)
- TOTP コピーのキーバインド `ctrl-t` は変更しない
- 両画面のヘッダー文言、フィールド選択画面の行ラベル(`ユーザー名をコピー (alt-u)` 等)の表記を新キーバインドに更新する
- README.md のキーバインド説明を更新し、Terminal.app の「Use Option as Meta Key」設定に関する注記を削除する

## Capabilities

### New Capabilities

(なし)

### Modified Capabilities

- `vault-item-search`: 検索画面での直接コピーのキーバインドに関する記述(`alt-u`/`alt-p` 等の具体キー名)を更新
- `credential-clipboard-copy`: キーバインドによるショートカットコピーの要件・シナリオで使用しているキー名(`alt-u`/`alt-p` → `ctrl-o`/`ctrl-r`)を更新。fzf のネイティブ操作(`ctrl-p`/`ctrl-n`/`ctrl-u`)を侵害しないという既存要件は変更なしで維持する

## Impact

- `lib/search.sh`: `--bind`/`--header` のキー指定
- `lib/fields.sh`: `--bind`/`--header` のキー指定、`bwqa_build_field_rows` のラベル文字列
- `README.md`: キーバインド説明、Meta キー設定に関する注記の削除
- `openspec/specs/vault-item-search/spec.md`, `openspec/specs/credential-clipboard-copy/spec.md`: delta spec 経由での更新
- 既存ユーザーへの周知が必要(README 更新が実質的な告知を兼ねる)

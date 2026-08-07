## Why

検索画面(インクリメンタルサーチ中)でアイテムを絞り込んでも、ユーザー名/パスワード/TOTP をコピーするにはいったんフィールド選択画面へ遷移する必要がある(issue #3)。よく使うアイテムほど「検索 → 即コピー」の1アクションで完結させたい。

## What Changes

- 検索画面の fzf に `ctrl-u`(ユーザー名)/`ctrl-p`(パスワード)/`ctrl-t`(TOTP)の `execute-silent` バインドを追加し、フィールド選択画面を経由せず直接コピーできるようにする
- コピー後も検索画面は閉じず、続けて別アイテムを検索・コピーできるようにする(フィールド選択画面の連続コピーと同じ挙動)。画面を閉じるのは Esc(既存の Enter によるフィールド選択画面への遷移は維持)
- 検索画面に `--border=rounded` と `--border-label` を追加し、フィールド選択画面と同じ状態ファイル(`BWQA_COPY_STATUS_FILE`)経由でコピー結果(成功/フィールド未設定/失敗)を表示する
- 検索画面の `--header` を更新し、新しいキーバインドを案内する
- `ctrl-u`/`ctrl-p`/`ctrl-t` 実行時、ハイライト中の行の item id を `{1}` でコマンド文字列に埋め込んで `__copy-field` に渡す(session token は従来どおりコマンド文字列に埋め込まず、画面起動前の `export BW_SESSION` で子プロセスへ継承させる。item id は非秘匿な識別子であり、この扱いの違いを `lib/fields.sh` 冒頭のコメントに明記する)

## Capabilities

### New Capabilities
(なし)

### Modified Capabilities
- `vault-item-search`: 検索画面にキーバインドによる直接コピー機能を追加する要件を追加する。既存の「Enter でフィールド選択画面に遷移する」要件はそのまま維持する
- `credential-clipboard-copy`: 「キーバインドによるショートカットコピー」「連続コピー」の各要件を、フィールド選択画面に限定せず検索画面にも適用されるよう拡張する
- `copy-feedback`: コピー結果のフィードバック表示を、フィールド選択画面に限定せず検索画面にも適用されるよう拡張する

## Impact

- `lib/search.sh`: `bwqa_run_search_screen()` にキーバインド・border-label・header 更新を追加
- `lib/fields.sh`: `bwqa_copy_field_internal()` の呼び出し規約は変更しないが、item id をコマンド文字列に埋め込む扱いについてコメントを更新する。検索画面・フィールド選択画面で重複する bind 構築ロジックを共通化するかは実装時に判断する
- `test/lib/search.bats`: fzf 対話画面自体はスコープ外の方針を踏襲しつつ、新規に追加するロジック(あれば)のテストを検討する
- README: 操作方法の説明に検索画面からの直接コピーを反映する

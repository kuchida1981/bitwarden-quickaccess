## Context

`lib/search.sh` の `bwqa_run_search_screen()` は `id \t label` の2カラムを `fzf --with-nth=2` に渡すだけで、コピー用の `--bind` は持たない。コピーは `lib/fields.sh` の `bwqa_run_field_screen()` に遷移してから、`enter`/`ctrl-p`/`ctrl-u`/`ctrl-t` の `execute-silent` バインド経由で `bwqa_copy_field_internal()`(`__copy-field` サブコマンドとして自プロセスを再帰起動)を呼ぶことで行う。

直前の change (`field-copy-feedback`) で、この issue (#3) は明示的に Non-Goal として先送りされている。

> 検索画面からの直接コピー(issue #3)は対象外。別 change で扱う。ただし `bwqa_copy_field_internal()` の呼び出し規約(item_id を引数からも受け取れるようにする等)を破壊しない形にはしておく

`bwqa_copy_field_internal()` はフィールドの存在有無を事前に知らず、`bw get <field> <item_id>` を実行して結果(成功/空値/コマンド失敗)を `$BWQA_COPY_STATUS_FILE` に書くだけの汎用ロジックである(fields.sh:102-153)。フィールド選択画面側の `has_username` 等によるフィルタは表示行を絞るためだけのものであり、コピー処理自体には影響しない。そのため検索画面の行から item id さえ渡せれば、フィールド選択画面と同じ `__copy-field` をそのまま流用できる。

`lib/fields.sh` 冒頭には次の設計原則がコメントとして明記されている。

> アイテム ID / session token はコマンド文字列に埋め込まず、環境変数(BWQA_ITEM_ID / BW_SESSION)経由で子プロセスに継承させる

フィールド選択画面では item_id が画面滞在中ずっと固定のため、fzf 起動前の `export BWQA_ITEM_ID="$item_id"` で一度だけ渡せば足りる。しかし検索画面では item_id はハイライト中の行によって変わるため、fzf 起動前の一括 export では対応できない。

## Goals / Non-Goals

**Goals:**
- 検索画面で `ctrl-u`/`ctrl-p`/`ctrl-t` を押すと、フィールド選択画面に遷移せず、ハイライト中のアイテムのユーザー名/パスワード/TOTP を直接クリップボードへコピーする
- コピー後も検索画面を終了させず、ユーザーが明示的な終了操作(Esc)を行うまで検索・コピーを繰り返せるようにする(フィールド選択画面の連続コピーと同じ挙動)
- コピー結果(成功/フィールド未設定/`bw` コマンド失敗)を、フィールド選択画面と同じ `--border-label` の仕組みで検索画面上にも表示する
- 既存の Enter によるフィールド選択画面への遷移は変更しない(全フィールドを一覧したい場合や、キーバインドを覚えていない場合の経路として維持する)

**Non-Goals:**
- フィールド選択画面自体の廃止・置き換えは行わない
- `bwqa_copy_field_internal()` の内部ロジック(3パターンのメッセージ区別)の変更は行わない。呼び出し方法のみを拡張する
- `bw` コマンドのタイムアウト制御・リトライ・再認証ロジックの変更は対象外
- 検索結果が0件の状態でキーバインドを押した場合の専用フィードバック(border-label へのメッセージ表示)は本 change のスコープ外とする。現状の `bwqa_copy_field_internal()` の早期ガードはエラーログにのみ記録し画面上には何も表示しないが、この挙動は変更しない(実質無反応になるだけで、誤ってコピーが行われるわけではないため許容する)

## Decisions

- **item id は `{1}` でコマンド文字列に埋め込む**: フィールド選択画面の `enter` バインドが既にフィールド名(`username`/`password`/`totp`)を `{1}` でコマンド文字列に埋め込んでいる前例がある。item id は vault アイテムの UUID であり、それ単体では何の情報も取得できない非秘匿な識別子である(session token とは秘匿性のレベルが異なる)。よって「コマンド文字列に埋め込まない」という原則は実質的に session token(実際の認証情報)を対象にしたものと整理し、item id は `{1}` 埋め込みを許容する。`lib/fields.sh` 冒頭のコメントをこの区別が分かるように更新する。
  - 代替案として状態ファイル経由(ハイライト変更のたびに現在の item id を一時ファイルへ書き、`__copy-field` 側で読む)も検討したが、`execute-silent` を2段に連結する必要があり実装・検証コストが上がる割に得られる安全性の向上が小さいため採用しない
- **session token は従来どおり環境変数経由**: `bwqa_run_search_screen()` の fzf 呼び出し全体を subshell で包み、起動前に `export BW_SESSION="$BWQA_SESSION"` する。検索画面滞在中は session が変わらないため、フィールド選択画面と同じパターンがそのまま使える
- **フィードバック表示はフィールド選択画面と同じ状態ファイル方式を再利用**: 新規の状態ファイルは作らず、既存の `$BWQA_COPY_STATUS_FILE` と `transform-border-label(cat "$BWQA_COPY_STATUS_FILE")` をそのまま検索画面にも設定する。表示先の意味(直近のコピー操作結果)が画面をまたいでも変わらないため、専用ファイルに分ける理由がない
- **`--border=rounded` を検索画面にも追加**: `--border-label` は `--border` が有効な場合のみ描画される。現在の検索画面は `--border` を指定していないため追加する。レイアウトへの影響は実装時に目視確認する
- **コピー後は画面を閉じない**: `ctrl-u`/`ctrl-p`/`ctrl-t` は `execute-silent` バインドとして追加し、`enter`/`esc` の既存動作(選択確定/終了)には触れない。これによりフィールド選択画面の「連続コピー」と一貫した体験になる
- **重複ロジックの共通化は本 change では見送る**: `export BW_SESSION` + border-label transform + execute-silent バインド構築は `search.sh` と `fields.sh` で似た形になるが、両者は「固定の item id に対して複数フィールドを扱う」(fields.sh)と「複数の item id に対して固定のフィールドを扱う」(search.sh)という違いがあり、素直な共通化がやや不自然になる。重複は小さく、無理に抽象化するとかえって読みにくくなるため、今回は許容し将来3箇所目の需要が出た時点で再検討する

## Risks / Trade-offs

- [item id を `{1}` でコマンド文字列に埋め込むことで、`ps` 等から一時的に item id が見える] → item id は非秘匿情報であり、フィールド名も既に同じ方法で埋め込まれている前例があるため許容する
- [検索結果0件時にキーバインドを押しても無反応に見える] → 実際には何もコピーされないため誤動作ではない。将来的にフィードバックが必要になれば別 change で対応する
- [`--border` 追加による検索画面のレイアウト変化(表示行数の減少等)] → 実装時に実機で目視確認し、`--height` の調整要否を判断する
- [`search.sh` と `fields.sh` 間のロジック重複が今後さらに増える] → 3箇所目の重複が発生した時点で共通化を検討する

## Open Questions

(なし)

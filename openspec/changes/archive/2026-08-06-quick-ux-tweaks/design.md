## Context

`lib/fields.sh` の `bwqa_build_field_rows()` は、フィールド選択画面の一覧をパスワード→ユーザー名→TOTP の順に並べている(コメント上は「Enter で先頭行を選ぶとパスワードがコピーされる」ことを意図した設計だったが、実運用では逆に事故りやすいと判明した)。

また `lib/session.sh` の `bwqa_unlock()`、`lib/search.sh` の `bwqa_fetch_items()`、`lib/fields.sh` の `bwqa_get_item_summary()` / `bwqa_copy_field_internal()` はいずれも `bw` CLI を同期的に呼び出しており、応答が返るまで画面に何も表示されない。`bwqa_log()`(`lib/common.sh`)は stderr にメッセージを出す既存のロギング関数で、`bwqa_bw()` の再認証時などですでに使われているパターンを踏襲できる。

## Goals / Non-Goals

**Goals:**
- フィールド選択画面の表示順序をユーザー名→パスワード→TOTP に変更し、spec 上の要件として明文化する
- `bw` CLI 呼び出し(session unlock / vault一覧取得 / アイテム詳細取得 / フィールド値取得)の直前に、`bwqa_log` でローディングメッセージを stderr に出す

**Non-Goals:**
- fzf の `reload` 等を使った非同期プログレッシブ表示(issue #5 の実装メモにある発展案)は対象外。まずは最小の対応(呼び出し前のメッセージ出力)にとどめる
- フィールドコピー成功時のフィードバック表示(issue #4)、検索画面からの直接コピー(issue #3)は別 change で扱う

## Decisions

- **表示順序の変更方法**: `bwqa_build_field_rows()` 内の jq 配列の並びをユーザー名→パスワード→TOTP に入れ替えるのみ。`enter:execute-silent(... __copy-field {1})` は「現在ハイライトしている行」をコピーする実装のため、キーバインド(`ctrl-p`/`ctrl-u`/`ctrl-t`)自体は変更不要で、Enter の既定コピー対象が変わるだけ
- **ローディングメッセージの出し方**: 新しい共通ヘルパーは作らず、既存の `bwqa_log "<メッセージ>"` を各 `bw` 呼び出し関数の直前に1行追加する形で統一する。理由: 呼び出し箇所は4箇所のみで、いずれも「関数の先頭で1回ログを出す」という同一パターンに収まるため、抽象化の必要がない
- **メッセージの文言**: 「vaultを読み込み中...」「アイテム情報を取得中...」等、処理内容が分かる短い日本語文言をそれぞれの呼び出しに合わせて個別に書く(汎用的な「処理中...」1種類にはしない。どの操作で待たされているかをユーザーが区別できるようにするため)

## 実装中に判明した制約(コードレビューで検出)

`bwqa_copy_field_internal()` は `lib/fields.sh` の `bwqa_run_field_screen()` 内で fzf の `execute-silent(...)` バインド経由でのみ呼び出される(`enter`/`ctrl-p`/`ctrl-u`/`ctrl-t`)。fzf の man page(COMMAND EXECUTION セクション)によれば、`execute-silent` は「画面切り替えをせずに静かにコマンドを実行する」動作であり、実際に tmux 上で検証したところ、子プロセスが stderr に書き込んだ内容は画面上に一切表示されないことを確認した(`/dev/tty` への直接書き込みは表示はされるものの、プロンプト行を上書きする形で汚れるためクリーンな表示にならない)。

そのため、`bwqa_copy_field_internal()` への `bwqa_log` 追加は当初 tasks.md 2.4 で計画したが、**実装を見送った**。この呼び出し経路でローディング表示を行うには、fzf のヘッダーを動的に書き換える仕組み(`change-header`/`transform-header` 等)が必要であり、これは issue #4(コピー成功フィードバック)のために別 change で扱う設計判断(fzf最低バージョン要件の見直しを含む)と同じ基盤を必要とする。本 change のスコープ(最小限の stderr メッセージ追加)には収まらないため、specs/loading-feedback/spec.md からも該当シナリオを削除した。

## Risks / Trade-offs

- [ローディングメッセージが `bwqa_log` 経由(stderr)で出ることで、`fields.sh` の「機密情報を標準出力に出力しない」要件との境界が曖昧に見える可能性がある] → メッセージ文言に機密情報(アイテム名やフィールド値)を含めず、固定文言のみを出力することで担保する
- [表示順序変更により、既存ユーザーが「Enter = パスワードコピー」という操作に慣れている場合、事故的にユーザー名がコピーされる遷移期間が生じる] → CHANGELOG/README相当の記載は本 change のタスクで検討するが、破壊的変更ではなく UX 改善のため許容する

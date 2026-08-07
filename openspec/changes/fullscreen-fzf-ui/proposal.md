## Why

現状、ロード中メッセージは標準エラー出力に1行ずつ流れていき、コピー操作(`bw` CLI 呼び出し)中は fzf の `execute-silent` の仕様上ユーザーに進行状況が一切見えない(`openspec/specs/loading-feedback/spec.md` で明示的に「別 change で扱う」とされていた箇所)。issue #16 の探索の結果、fzf 自体が持つ機能(フルスクリーンモード、`every(N)` タイマーイベント、`bg-transform-*` 非同期アクション)でこれを解決できることが分かった。ただし `every(N)` は fzf v0.73.0 で導入された比較的新しい機能であり、これを使うには最低要件の引き上げが必要になる。

## What Changes

- 検索画面・フィールド選択画面を、`--height=80%` を外した fzf ネイティブのフルスクリーン表示(alternate screen buffer)に変更する。オプトインフラグは設けず、常時この表示に置き換える。**BREAKING**: 画面表示中、ターミナルのスクロールバックが一時的に隠れる挙動になる(fzf 終了時に元の画面へ復元される)。
- フィールドコピー処理(`__copy-field` サブコマンド)を `execute-silent` からバックグラウンド実行に変更し、コピー処理中は fzf の `every(N)` + `bg-transform-border-label` によりボーダーラベルへスピナー(進行中インジケーター)を表示する。処理完了後は従来通り結果メッセージ(成功/フィールド未設定/失敗)を表示する。
- fzf の最低要件を `0.37.0` から `0.73.0` に引き上げる(`bwqa_check_fzf_version` の `required` 定数を変更)。fzf バージョンによる機能の出し分け(フォールバック分岐)は行わない。
- README(日本語版・英語版)の必要要件の記載を更新する。
- 起動直後の `bw unlock`(マスターパスワード入力)フェーズのフルスクリーン化、および実行中のセッション切れへの対応は本 change のスコープ外とし、issue #32 に切り出し済み。

## Capabilities

### New Capabilities
(なし)

### Modified Capabilities
- `vault-item-search`: 検索画面の表示方式をフルスクリーン(alternate screen buffer)に変更する要件を追加
- `credential-clipboard-copy`: フィールド選択画面の表示方式をフルスクリーン(alternate screen buffer)に変更する要件を追加
- `copy-feedback`: コピー処理が進行中である間、結果が確定するまでの間にスピナー等の進行状況表示を行う要件を追加
- `loading-feedback`: `execute-silent` 経由のコピー処理中の表示を対象外としていた記述を、本 change でのスピナー対応(`copy-feedback` でカバー)を踏まえて更新
- `environment-preflight`: fzf の最低バージョン確認(0.73.0 以上)に関する要件を新規に明文化

## Impact

- 影響コード: `lib/search.sh`(`bwqa_run_search_screen`)、`lib/fields.sh`(`bwqa_run_field_screen`、`bwqa_copy_field_internal`)、`lib/preflight.sh`(`bwqa_check_fzf_version`)
- 影響ドキュメント: `README.md`、`README.ja.md`(fzf バージョン要件の記載)
- 影響テスト: `test/lib/*.bats` のうち、fzf 起動オプション・バージョンチェックに関するテスト
- 依存関係: fzf の必須バージョンが `0.37.0` → `0.73.0` に上がるため、古い fzf を使っているユーザーはアップグレードが必要になる(preflight で明示的にエラー終了し、案内を表示するため無言の失敗にはならない)

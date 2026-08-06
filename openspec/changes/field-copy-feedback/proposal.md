## Why

フィールド選択画面でコピー操作(Enter / ctrl-u / ctrl-p / ctrl-t)を行っても、成功・失敗が画面上に一切表示されない。fzf の `execute-silent` は子プロセスの stdout/stderr をターミナルに表示しない仕様であるため、既存の `bwqa_log`(stderr 出力)では対応できないことが直前の change (`quick-ux-tweaks`) の実装時に判明し、fzf のヘッダー/ラベル書き換えという別の仕組みが必要な課題として本 change に先送りされていた。

また、既存 spec `credential-clipboard-copy` の Requirement「フィールド選択によるクリップボードコピー」には「該当フィールドが存在しないアイテムを選んだ場合、フィールドが存在しない旨をユーザーに分かる形で示す」というシナリオが定義済みだが、これはリスト選択時にはフィールドが一覧から除外されることで実質的に発生しないケースを指しており、キーバインドショートカット(ctrl-u/p/t)で存在しないフィールドを指定した場合には実際には無反応(エラーログファイルへの記録のみ)で、要件を満たせていない。

## What Changes

- `bwqa_copy_field_internal()` が、コピーの成功/失敗結果を1行のメッセージとして状態ファイル(`BWQA_CACHE_DIR` 配下)に書き込むようにする(機密情報は含めない)
- フィールド選択画面の fzf 起動オプションに `--border-label` を追加し、コピー実行後に `transform-border-label` で状態ファイルの内容を反映する。`--header` の操作説明はそのまま維持する
- フィールドの値が空(=フィールドが存在しない)だった場合も、成功時と区別できるメッセージを表示する
- フィードバックの表示は自動では消さず、次のコピー操作や画面遷移まで表示し続ける
- `bwqa_check_fzf_version()` の最低バージョン要件を `0.35.0` → `0.37.0` に引き上げる(`transform-border-label` / `change-border-label` は fzf 0.37.0 で追加された機能のため)。README の必要環境の記載もあわせて更新する

## Capabilities

### New Capabilities
- `copy-feedback`: フィールド選択画面でのコピー操作(成功/失敗)を、画面上で視覚的に確認できるようにするフィードバック表示の仕組み

### Modified Capabilities
- `credential-clipboard-copy`: 「該当フィールドが存在しないアイテムを選んだ場合」のシナリオを、キーバインドショートカット(ctrl-u/p/t)経由で存在しないフィールドを指定したケースも明確にカバーするよう明文化する(リスト選択時の除外だけでなく、キーバインドでの直接指定時にもフィードバックで示すことを要件化する)

## Impact

- `lib/fields.sh`: `bwqa_run_field_screen()`(`--border-label` 追加、`--bind` に `transform-border-label` を追加)、`bwqa_copy_field_internal()`(状態ファイルへの結果書き込み)
- `lib/common.sh`: 状態ファイルパスの定数追加(既存の `BWQA_ERROR_LOG_FILE` と同様のパターン)
- `lib/preflight.sh`: `bwqa_check_fzf_version()` の最低バージョンを `0.37.0` に変更
- `README.md`: 必要な `fzf` バージョンの記載を更新
- `test/lib/fields.bats`: 状態ファイル書き込み・フィードバック文言に関するテストを追加

## 1. fzf 最低要件の引き上げ

- [ ] 1.1 `lib/preflight.sh` の `bwqa_check_fzf_version` にある `required="0.37.0"` を `"0.73.0"` に変更する
- [ ] 1.2 `test/lib/preflight.bats` のバージョン境界値に関する既存テストを新しい要求バージョンに合わせて更新する

## 2. コピー進行状況の共通基盤

- [ ] 2.1 `lib/common.sh` にコピー処理中を示すロックファイルパス定数(例: `BWQA_COPY_LOCK_FILE`)を追加する
- [ ] 2.2 `lib/fields.sh` の `bwqa_copy_field_internal` の冒頭でロックファイルを作成し、`trap ... EXIT` で関数の全終了経路(成功・失敗・不正フィールド)においてロックファイルが確実に削除されるようにする
- [ ] 2.3 `lib/i18n/ja.sh` / `lib/i18n/en.sh` に、コピー処理中を示すメッセージ(例: 「コピー中...」/「Copying...」)を追加する
- [ ] 2.4 `bin/bw-quickaccess` に `__copy-status` サブコマンドを追加する。ロックファイルが存在する場合は経過時間からスピナーのフレームを算出しコピー中メッセージと合わせて標準出力へ、存在しない場合は `BWQA_COPY_STATUS_FILE` の内容をそのまま標準出力へ出力する

## 3. 検索画面のフルスクリーン化とスピナー結線

- [ ] 3.1 `lib/search.sh` の `bwqa_run_search_screen` 内 fzf 起動オプションから `--height=80%` を外す
- [ ] 3.2 `ctrl-o`/`ctrl-r`/`ctrl-t` バインドの `execute-silent(...)` をバックグラウンドジョブ化(末尾に `&` を付与)し、既存の `: {1}` による0件マッチ時スキップの挙動を維持したまま、直後に `__copy-status` を呼ぶ `transform-border-label` へ差し替える
- [ ] 3.3 `every(N):bg-transform-border-label(... __copy-status)` バインドを追加する(N は実装時に体感で調整、0.1〜0.2秒程度を想定)

## 4. フィールド選択画面のフルスクリーン化とスピナー結線

- [ ] 4.1 `lib/fields.sh` の `bwqa_run_field_screen` 内 fzf 起動オプションから `--height=80%` を外す
- [ ] 4.2 `enter`/`ctrl-o`/`ctrl-r`/`ctrl-t` バインドの `execute-silent(...)` をバックグラウンドジョブ化し、直後に `__copy-status` を呼ぶ `transform-border-label` へ差し替える
- [ ] 4.3 `every(N):bg-transform-border-label(... __copy-status)` バインドを追加する(3.3 と同じ間隔値を使用する)

## 5. ドキュメント更新

- [ ] 5.1 `README.md` / `README.ja.md` の必要要件セクションで fzf の最低バージョン記載を `0.73.0` に更新する
- [ ] 5.2 `README.md` / `README.ja.md` に、検索画面・フィールド選択画面がフルスクリーン表示になる旨(スクロールバックが一時的に隠れる挙動を含む)を追記する

## 6. テスト

- [ ] 6.1 `test/lib/preflight.bats` に、要求バージョン未満・要求バージョン以上・バージョン判定不能の3ケースを網羅するテストがあることを確認し、不足があれば追加する
- [ ] 6.2 `__copy-status` サブコマンドについて、ロックファイルが存在する場合としない場合それぞれで出力が切り替わることを検証するテストを追加する(`test/lib/fields.bats` または新規ファイル)
- [ ] 6.3 `test/lib/search.bats` / `test/lib/fields.bats` の既存テストのうち、fzf 起動コマンド文字列を検証しているものを更新し、`--height` オプションが含まれないこと・`every(...)` バインドが含まれることをアサーションに追加する
- [ ] 6.4 `bash -n` / `shellcheck` / `bats test/lib/*.bats` を実行し、全て成功することを確認する

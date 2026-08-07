## Why

CLI が出力するメッセージ(`bwqa_log`/`bwqa_die` 経由のエラー・警告・案内、fzf のプロンプト・プレビュー文言)と README が日本語固定になっており、英語話者のユーザーが利用しづらい(issue #15)。対応言語をまず日本語・英語の2言語に広げ、実行時のロケールに応じて自動で切り替わるようにする。

## What Changes

- `lib/i18n/ja.sh` / `lib/i18n/en.sh` を新設し、`BWQA_MSG_*` プレフィックスの変数でメッセージ文字列を言語別に定義する(連想配列は使わない。macOS 標準 bash 3.2 に非対応のため)
- 起動時に `BWQA_LANG` 環境変数(明示指定)→ `LANG`/`LC_ALL` の順で言語を判定し、対応する `lib/i18n/<lang>.sh` を `source` する。判定できない・対応言語ファイルが存在しない場合は `en` にフォールバックする
- `lib/*.sh` 内の `bwqa_log`/`bwqa_die` 呼び出し(現状12箇所)のメッセージ文字列を `BWQA_MSG_*` 変数参照に置き換える
- fzf のプロンプト文言・プレビューペインの文言も同様に i18n 化する
- `README.md` を英語に書き換え、`README.ja.md` に既存の日本語 README を移し、両ファイルの冒頭に相互リンクを設置する
- gettext/ngettext は不採用とする(理由: macOS 標準 bash 3.2 は連想配列非対応、`gettext` コマンドは macOS 非標準で Homebrew 経由の追加依存になる、`.po`→`.mo` のビルドステップが既存の無ビルド配布モデルと合わない)

## Capabilities

### New Capabilities

- `message-localization`: CLI が出力するユーザー向けメッセージ(ログ・エラー・fzf 文言)を日本語・英語で切り替え可能にし、`BWQA_LANG` またはロケール環境変数に応じて実行時に言語を選択する

### Modified Capabilities

(なし。メッセージの言語切り替えは新規capabilityであり、既存capabilityの要求仕様(振る舞い)自体は変更しない)

## Impact

- 影響コード: `lib/common.sh`(`bwqa_log`/`bwqa_die` 呼び出し元)、`lib/search.sh`・`lib/preflight.sh`・`lib/fields.sh`・`lib/session.sh`(メッセージ文字列箇所、fzf 起動オプション)、`bin/bw-quickaccess`(起動時の i18n ファイル source 追加)
- 新規ファイル: `lib/i18n/ja.sh`、`lib/i18n/en.sh`
- ドキュメント: `README.md`(英語化)、`README.ja.md`(新規、既存日本語版を移設)
- 新規外部依存なし(既存の `source` パターンのみを使用)
- 後方互換性: `BWQA_LANG` 未設定時はロケール判定にフォールバックするため、既存ユーザーの挙動(日本語ロケール環境なら日本語表示)は変わらない

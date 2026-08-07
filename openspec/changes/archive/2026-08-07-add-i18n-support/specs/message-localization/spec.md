## ADDED Requirements

### Requirement: 言語ファイルによるメッセージ管理
システムは、ユーザー向けメッセージ文字列を `lib/i18n/ja.sh` と `lib/i18n/en.sh` に `BWQA_MSG_*` プレフィックスの変数として定義し、連想配列(`declare -A`)を使用してはならない(SHALL NOT)。

#### Scenario: 日本語メッセージファイルが存在する
- **WHEN** `lib/i18n/ja.sh` を読み込む
- **THEN** `bwqa_log`/`bwqa_die` および fzf の `--prompt`/`--header` が参照する全ての `BWQA_MSG_*` 変数が日本語文字列として定義されている

#### Scenario: 英語メッセージファイルが存在する
- **WHEN** `lib/i18n/en.sh` を読み込む
- **THEN** `bwqa_log`/`bwqa_die` および fzf の `--prompt`/`--header` が参照する全ての `BWQA_MSG_*` 変数が英語文字列として定義されている

#### Scenario: bash 3.2 環境でも動作する
- **WHEN** macOS 標準の `/bin/bash`(3.2系、連想配列非対応)で `bw-quickaccess` を実行する
- **THEN** 言語ファイルの読み込みとメッセージ表示がエラーなく完了する

### Requirement: 言語の自動判定と明示指定
システムは、起動時に `BWQA_LANG` 環境変数を最優先で参照し、未設定の場合は `LC_ALL` → `LANG` の順にロケール文字列から言語を判定し、`ja` で始まらない場合は `en` を選択しなければならない(SHALL)。

#### Scenario: BWQA_LANG による明示指定
- **WHEN** `BWQA_LANG=en` が設定された状態で起動する(`LANG` が `ja_JP.UTF-8` であっても)
- **THEN** 英語メッセージが表示される

#### Scenario: LANG からの自動判定(日本語ロケール)
- **WHEN** `BWQA_LANG` が未設定で `LANG=ja_JP.UTF-8` が設定された状態で起動する
- **THEN** 日本語メッセージが表示される

#### Scenario: LANG からの自動判定(非日本語ロケール)
- **WHEN** `BWQA_LANG` が未設定で `LANG=en_US.UTF-8` が設定された状態で起動する
- **THEN** 英語メッセージが表示される

#### Scenario: ロケール未設定時のフォールバック
- **WHEN** `BWQA_LANG`・`LC_ALL`・`LANG` のいずれも未設定、または不正な値である状態で起動する
- **THEN** 英語メッセージが表示される

#### Scenario: 未対応言語ファイルへのフォールバック
- **WHEN** 判定された言語に対応する `lib/i18n/<lang>.sh` が存在しない
- **THEN** `lib/i18n/en.sh` が読み込まれる

### Requirement: 既存メッセージ出力箇所の網羅的な移行
システムは、`bwqa_log`/`bwqa_die` に直接渡される日本語リテラル文字列、および `lib/search.sh`・`lib/fields.sh` の fzf `--prompt`/`--header` オプションの日本語文言を、すべて `BWQA_MSG_*` 変数参照に置き換えなければならない(SHALL)。

#### Scenario: エラーメッセージの言語切り替え
- **WHEN** `BWQA_LANG=en` の状態で必須コマンド(`bw`/`jq`/`fzf`)が見つからずに `bwqa_die` が呼ばれる
- **THEN** エラーメッセージが英語で表示される

#### Scenario: fzf ヘッダーの言語切り替え
- **WHEN** `BWQA_LANG=en` の状態で vault アイテム検索画面(fzf)を起動する
- **THEN** `--header` に表示されるキーバインド説明文が英語で表示される

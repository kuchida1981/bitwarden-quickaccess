## ADDED Requirements

### Requirement: 純粋ロジック・分岐ロジックの単体テスト
`lib/common.sh` の `bwqa_version_ge()`、`lib/session.sh` の `bwqa_session_ttl_expired()`、`lib/search.sh` の `bwqa_fetch_items()`、`lib/fields.sh` の `bwqa_build_field_rows()` / `bwqa_get_item_summary()` / `bwqa_copy_field_internal()`、`lib/preflight.sh` のコマンド有無・バージョン・OS/表示サーバー判定ロジックについて、bats-core による単体テストが存在しなければならない(SHALL)。

#### Scenario: バージョン比較の桁上がりを検証する
- **WHEN** `bwqa_version_ge "0.36.0" "0.35.0"` および `bwqa_version_ge "0.9.0" "0.35.0"` を実行する
- **THEN** 前者は真(0)を返し、後者は偽(非0)を返す

#### Scenario: session TTL 判定の境界値を検証する
- **WHEN** issued-at ファイルに `BWQA_SESSION_TTL_SECONDS` ちょうど・1秒未満・1秒超過の経過時間を書き込んだ状態で `bwqa_session_ttl_expired` を実行する
- **THEN** TTL 未満では偽、TTL 以上では真を返す

#### Scenario: issued-at ファイルが存在しない場合は期限切れ扱いにする
- **WHEN** issued-at ファイルが存在しない状態で `bwqa_session_ttl_expired` を実行する
- **THEN** 真(期限切れ)を返す

#### Scenario: vault アイテム一覧を type==1 のみに絞り込んで整形する
- **WHEN** ログインタイプ以外を含む `bw list items` 相当の JSON を返す `bwqa_bw` スタブを使って `bwqa_fetch_items` を実行する
- **THEN** 戻り値の JSON には type==1 のアイテムのみが id/label 形式で含まれる

#### Scenario: フィールド有無に応じてコピー候補行を生成する
- **WHEN** password のみ・username のみ・全フィールドありなど異なる has_* の組み合わせのサマリ JSON を渡して `bwqa_build_field_rows` を実行する
- **THEN** has_* が真のフィールドのみが行として出力され、password が存在する場合は先頭行になる

### Requirement: 外部コマンド依存箇所のモック方式
`lib/*.sh` の外部コマンド依存を持つ関数のテストは、対象関数の関心事に応じてモック方式を使い分けなければならない(SHALL)。内部ラッパー関数(`bwqa_bw()` 等)を介して外部コマンドを呼ぶ箇所は、テスト内での関数スタブ(関数の再定義)によってモックする。コマンドの有無・バージョンの検出そのものを検証する箇所(`bwqa_check_core_tools`、`bwqa_check_fzf_version`、`bwqa_detect_clipboard_cmd`)、および内部ラッパーを介さず外部コマンドを直接呼ぶ箇所(`bwqa_copy_field_internal` の `bw get` 呼び出し)は、一時 PATH ディレクトリに置いたダミー実行ファイルによってモックする。

#### Scenario: 必須コマンド不足時にエラー終了する
- **WHEN** PATH 上に `jq` が存在しないダミー環境で `bwqa_check_core_tools` を実行する
- **THEN** 非0で終了し、不足コマンド名を含むエラーメッセージがログに出力される

#### Scenario: fzf バージョンが要件未満の場合にエラー終了する
- **WHEN** `fzf --version` が `0.34.9` を返すダミー `fzf` を PATH に置いた状態で `bwqa_check_fzf_version` を実行する
- **THEN** 非0で終了し、検出バージョンと必要バージョンを含むエラーメッセージがログに出力される

#### Scenario: フィールド値取得失敗時にエラーログへ記録する
- **WHEN** `bw get password` が空文字を返すダミー `bw` を PATH に置いた状態で `bwqa_copy_field_internal password` を実行する
- **THEN** 非0で終了し、`BWQA_ERROR_LOG_FILE` に field 名とアイテム ID を含む失敗記録が追記される

### Requirement: CI による構文チェック・静的解析・テスト実行の自動化
リポジトリへの push および pull request に対し、GitHub Actions ワークフローが `macos-latest` と `ubuntu-latest` の両方で、`bash -n` による構文チェック・`shellcheck` による静的解析・bats テストの実行を行わなければならない(SHALL)。いずれかのチェックが失敗した場合、ワークフロー全体は失敗ステータスになる(SHALL)。

#### Scenario: 構文エラーを含むコミットで CI が失敗する
- **WHEN** `lib/*.sh` のいずれかに `bash -n` で検出可能な構文エラーを含む変更を push する
- **THEN** `macos-latest` / `ubuntu-latest` 双方のジョブが失敗ステータスで終了する

#### Scenario: テスト失敗を含むコミットで CI が失敗する
- **WHEN** 既存の bats テストのいずれかが失敗する変更を push する
- **THEN** 該当ランナーのジョブが失敗ステータスで終了する

#### Scenario: OS 判定ロジックを両 OS で検証する
- **WHEN** CI が `macos-latest` と `ubuntu-latest` の両方で `preflight.bats` を実行する
- **THEN** 各ランナーの実 OS(`uname` の実結果)に対応する分岐(macOS 判定 / Linux 判定)がそれぞれのランナー上で検証される

# release-packaging

## Purpose

TBD - `bin/` と `lib/*.sh` を単一の自己完結した実行可能ファイルにバンドルし、GitHub Release 公開時に自動でアセットとして添付する。

## Requirements

### Requirement: 単一ファイルへのバンドル
ビルドスクリプト(`script/build.sh`)は `bin/bw-quickaccess` と `lib/*.sh` を、現在の起動時 `source` 連鎖と同じ順序(`common.sh` → `preflight.sh` → `clipboard.sh` → `session.sh` → `search.sh` → `fields.sh`)で連結し、単一の自己完結した実行可能スクリプト(`bw-quickaccess`)を生成しなければならない(SHALL)。生成物は追加の `lib/` ディレクトリなしに単独で動作しなければならない(SHALL)。

#### Scenario: ビルドスクリプトを実行する
- **WHEN** `script/build.sh` を実行する
- **THEN** `bin/` と `lib/` に依存せず単独で実行可能な `bw-quickaccess` ファイルが生成される

#### Scenario: バンドル後の動作がソースコードと同一である
- **WHEN** バンドルされた `bw-quickaccess` を、`bin/` と `lib/` を同階層に置かずに実行する
- **THEN** `bin/bw-quickaccess` を clone した状態(`bin/`+`lib/` が兄弟ディレクトリ)で実行した場合と同一の挙動になる(vault アイテム検索・フィールドコピー・`lock` サブコマンド・fzf からの `__copy-field` 再帰呼び出しを含む)

### Requirement: リリース公開時のアセット自動添付
CI は GitHub Release が公開(`release` イベントの `published` タイプ)されたときに自動的に起動し、`script/build.sh` でバンドルをビルドした上で、公開された release にビルド成果物をアセットとして添付しなければならない(SHALL)。CI は release 自体を作成してはならない(SHALL NOT)。

#### Scenario: 新規タグを指定して release を公開する
- **WHEN** 人間が `gh release create v0.1.0 --generate-notes` を実行し release を公開する
- **THEN** CI が起動し、`v0.1.0` タグが指す commit からバンドルをビルドし、`v0.1.0` の release にアセットとして添付する

#### Scenario: 既存タグを指定して release を公開する
- **WHEN** 人間が既に push 済みのタグを指定して `gh release create` を実行し release を公開する
- **THEN** タグの push イベントが発火していない状態でも、release の公開イベントによって CI が起動し、バンドルがアセットとして添付される

#### Scenario: draft のまま release を保存する
- **WHEN** 人間が release を draft(未公開)として保存する
- **THEN** CI は起動せず、アセットの添付は行われない

### Requirement: バージョンの埋め込みとバージョン確認コマンド
ビルドスクリプト(`script/build.sh`)は環境変数 `VERSION` を受け取り、指定がある場合はそのバージョン文字列をバンドルに埋め込まなければならない(SHALL)。`VERSION` が指定されない場合、バンドルは実行時に `dev` をバージョンとして扱わなければならない(SHALL)。生成されたバンドルは `--version`/`-v` オプションを受け付け、`bw-quickaccess <埋め込まれたバージョン>` の形式で標準出力に出力し正常終了しなければならない(SHALL)。CI(`.github/workflows/release.yml`)はビルド時に、公開された release のタグ名を `VERSION` としてビルドスクリプトに渡さなければならない(SHALL)。

#### Scenario: リリースビルドでバージョンを埋め込む
- **WHEN** CI が `VERSION=v0.1.0` を指定して `script/build.sh` を実行する
- **THEN** 生成されたバンドルは `bw-quickaccess --version` 実行時に `bw-quickaccess v0.1.0` を出力する

#### Scenario: ローカルビルドでバージョン未指定の場合
- **WHEN** `VERSION` を指定せずに `script/build.sh` をローカルで実行する
- **THEN** 生成されたバンドルは `bw-quickaccess --version` 実行時に `bw-quickaccess dev` を出力する

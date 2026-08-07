## ADDED Requirements

### Requirement: curl ワンライナーによるインストール
システムは `curl -fsSL <install.sh の URL> | bash` の形式で実行できるインストールスクリプト(`install.sh`)を提供しなければならない(SHALL)。

#### Scenario: curl パイプで実行する
- **WHEN** ユーザーが `curl -fsSL <install.sh の URL> | bash` を実行する
- **THEN** `bw-quickaccess` 実行ファイルがローカルにインストールされ、`bw-quickaccess` コマンドとして実行可能になる

### Requirement: デフォルトはユーザー権限インストール
install.sh はデフォルトで root 権限を必要とせず、ユーザーのホームディレクトリ配下(`~/.local`)にインストールしなければならない(SHALL)。

#### Scenario: オプション無しで実行する
- **WHEN** ユーザーが `--prefix` を指定せずに install.sh を実行する
- **THEN** `bw-quickaccess` 実行ファイルが `~/.local/bin/bw-quickaccess` に配置され、`sudo` 等の昇格権限は要求されない

### Requirement: --prefix オプションによるインストール先変更
install.sh は `--prefix <path>` オプションを受け付け、指定されたディレクトリの `bin/` サブディレクトリに実行ファイルを配置しなければならない(SHALL)。

#### Scenario: --prefix を指定して実行する
- **WHEN** ユーザーが `install.sh --prefix /opt/bwqa` を実行する
- **THEN** `bw-quickaccess` 実行ファイルが `/opt/bwqa/bin/bw-quickaccess` に配置される

### Requirement: バージョン解決
install.sh はバージョン指定が無い場合は GitHub Release の最新版を、バージョン指定がある場合は該当タグの Release アセットをダウンロードしなければならない(SHALL)。バージョン指定が無い場合、GitHub API を呼び出さずに GitHub が提供する "latest" リダイレクト URL(`releases/latest/download/<asset>`)を使用しなければならない(SHALL)。

#### Scenario: バージョン指定無しで実行する
- **WHEN** ユーザーがバージョンを指定せずに install.sh を実行する
- **THEN** `https://github.com/<owner>/<repo>/releases/latest/download/bw-quickaccess` から最新版のバンドルがダウンロードされる

#### Scenario: バージョンを指定して実行する
- **WHEN** ユーザーが `install.sh --version v0.1.0` のようにバージョンを指定して実行する
- **THEN** `https://github.com/<owner>/<repo>/releases/download/v0.1.0/bw-quickaccess` から指定バージョンのバンドルがダウンロードされる

### Requirement: PATH 未設定時の警告表示
install.sh はインストール完了後、インストール先の `bin` ディレクトリが `PATH` 環境変数に含まれていない場合、警告メッセージと `PATH` に追加するためのコマンド例を表示しなければならない(SHALL)。シェルの設定ファイル(`.bashrc`/`.zshrc` 等)を自動的に編集してはならない(SHALL NOT)。

#### Scenario: インストール先が PATH に含まれていない
- **WHEN** インストール先の `bin` ディレクトリ(例: `~/.local/bin`)が現在の `PATH` に含まれていない状態で install.sh が完了する
- **THEN** `PATH` に追加するための `export PATH=...` コマンド例を含む警告メッセージが標準出力または標準エラー出力に表示され、いずれのシェル設定ファイルも変更されない

#### Scenario: インストール先が既に PATH に含まれている
- **WHEN** インストール先の `bin` ディレクトリが既に現在の `PATH` に含まれている状態で install.sh が完了する
- **THEN** PATH に関する警告メッセージは表示されない

### Requirement: install.sh 自体の依存
install.sh は自身の実行に `git` を必要としてはならない(SHALL NOT)。ダウンロードには `curl` のみを使用する。

#### Scenario: git が存在しない環境で実行する
- **WHEN** `git` コマンドがインストールされていない環境で install.sh を実行する
- **THEN** install.sh は正常に完了し、`bw-quickaccess` がインストールされる

### Requirement: install.sh の再実行によるアップデート
install.sh は専用のアップデートコマンドを持たず、再実行することで既存のインストールを上書き更新できなければならない(SHALL)。上書き前に既存のインストールが存在する場合、install.sh は `<インストール先>/bin/bw-quickaccess --version` を用いて現在のバージョンを取得し、更新後のバージョンと合わせて変更前後のバージョンをユーザーに表示しなければならない(SHALL)。既存のインストールが存在しない場合は、新規インストールである旨とインストールされたバージョンを表示しなければならない(SHALL)。

#### Scenario: 既存インストールがある状態で再実行する
- **WHEN** 既に `bw-quickaccess` がインストールされている状態(バージョン `v0.1.0`)で、最新版(バージョン `v0.2.0`)を対象に install.sh を再実行する
- **THEN** `$PREFIX/bin/bw-quickaccess` が `v0.2.0` のバンドルで上書きされ、`v0.1.0` から `v0.2.0` に更新した旨がユーザーに表示される

#### Scenario: 初回インストールの場合
- **WHEN** インストール先に `bw-quickaccess` が存在しない状態で install.sh を実行する
- **THEN** 新規にインストールされ、インストールされたバージョンがユーザーに表示される(アップデート表示にはならない)

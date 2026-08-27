## ADDED Requirements

### Requirement: Finder上の親しみやすい表示名
アプリの `productName`（Finder上のアプリ名・`.app` バンドル名の元）は、技術的な内部識別子とは独立した、親しみやすい表示名でなければならない(SHALL)。

#### Scenario: Finderでアプリを確認する
- **WHEN** ユーザーが `/Applications` またはインストール済みアプリの一覧でアプリを確認する
- **THEN** "Bitwarden Quick Access" という名前で表示される

### Requirement: 内部識別子の不変性
`productName` を変更する際、バンドル識別子(`tauri.conf.json` の `identifier`)および `Cargo.toml` のパッケージ名・バイナリ名は変更してはならない(SHALL NOT)。

#### Scenario: productName変更後も内部識別子が維持される
- **WHEN** `productName` が変更される
- **THEN** `tauri.conf.json` の `identifier` の値、および `Cargo.toml` の `package.name`・`[[bin]] name` の値は変更前と同じままである

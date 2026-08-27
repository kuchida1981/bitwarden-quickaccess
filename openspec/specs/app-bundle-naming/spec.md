# app-bundle-naming

## Purpose

アプリの `productName`（Finder上のアプリ名・`.app` バンドル名の元）を、技術的な内部識別子とは独立した、親しみやすい表示名にする。これにより、Finderやインストール済みアプリの一覧を見たユーザーが、アプリの用途を直感的に理解できるようにする。一方で、バンドル識別子や `Cargo.toml` のパッケージ名・バイナリ名といった内部識別子は、表示名の変更に影響されず不変であることを保証し、既存のインストール・アップデート・設定との互換性を維持する。

## Requirements

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

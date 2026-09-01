# dependency-update-automation

## Purpose

Cargo 依存関係と GitHub Actions のバージョン更新を Dependabot で自動検出し、CI 通過を条件に低リスクな更新(minor/patch)を自動マージすることで、依存関係の追従漏れを防ぐ。

## Requirements

### Requirement: 依存関係更新の自動検出
システムは `app/src-tauri` の cargo 依存関係と `.github/workflows` で使用される GitHub Actions を週次でチェックし、新しいバージョンが存在する場合は更新プルリクエストを作成しなければならない(SHALL)。

#### Scenario: cargo 依存関係の新バージョンが存在する
- **WHEN** `app/src-tauri` の Cargo 依存関係に新しいバージョンが公開されている
- **THEN** システムは週次スケジュール内に更新を提案するプルリクエストを作成する

#### Scenario: GitHub Actions の新バージョンが存在する
- **WHEN** `.github/workflows` で使用される GitHub Action に新しいバージョンが公開されている
- **THEN** システムは週次スケジュール内に更新を提案するプルリクエストを作成する

### Requirement: main へのマージにはCI成功が必須
main ブランチへのマージは、CI（`test` ジョブの macOS および Linux マトリクス）のステータスチェックが成功していることを必須条件としなければならない(SHALL)。

#### Scenario: main を対象にプルリクエストが作成・更新される
- **WHEN** main を対象とするプルリクエストが作成または更新される
- **THEN** CI(`test`)のマトリクス（macOS, Linux）ステータスチェックが実行され、すべてのマトリクスが成功するまでマージはブロックされる

### Requirement: 低リスクな依存関係更新の自動マージ
Dependabot が作成したプルリクエストのうち、`semver-patch` または `semver-minor` に分類される更新は、CI成功後に自動でマージされなければならない(SHALL)。

#### Scenario: patch/minor 更新がCIを通過する
- **WHEN** Dependabot のプルリクエストが `semver-patch` または `semver-minor` に分類され、CI のステータスチェックが成功する
- **THEN** システムはそのプルリクエストをマージコミット方式で自動的にマージする

#### Scenario: major 更新は自動マージされない
- **WHEN** Dependabot のプルリクエストが `semver-major` に分類される
- **THEN** システムはそのプルリクエストを自動マージせず、手動レビューのために残す

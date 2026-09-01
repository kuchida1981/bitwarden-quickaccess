## MODIFIED Requirements

### Requirement: main へのマージにはCI成功が必須
main ブランチへのマージは、CI（`test` ジョブの macOS および Linux マトリクス）のステータスチェックが成功していることを必須条件としなければならない(SHALL)。

#### Scenario: main を対象にプルリクエストが作成・更新される
- **WHEN** main を対象とするプルリクエストが作成または更新される
- **THEN** CI(`test`)のマトリクス（macOS, Linux）ステータスチェックが実行され、すべてのマトリクスが成功するまでマージはブロックされる

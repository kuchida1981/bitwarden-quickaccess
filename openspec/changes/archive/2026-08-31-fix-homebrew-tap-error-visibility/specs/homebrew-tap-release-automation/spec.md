## ADDED Requirements

### Requirement: PAT未設定時のスキップとPR作成失敗の可視化
`HOMEBREW_TAP_PAT` シークレットが未設定の場合、システムはHomebrew tap更新に関する一連のステップをスキップし、この既知の未設定状態単独でリリースワークフロー全体を失敗として扱ってはならない(SHALL NOT)。一方、`HOMEBREW_TAP_PAT` が設定されているにも関わらずtap更新PRの作成(`brew tap`/`brew trust`/`brew bump-cask-pr`)が失敗した場合、システムはリリースワークフロー全体を失敗として扱わなければならない(SHALL)。

#### Scenario: PAT未設定時はスキップされリリースワークフローは成功扱いになる
- **WHEN** `HOMEBREW_TAP_PAT` シークレットが未設定の状態でリリースワークフローが実行される
- **THEN** Homebrew tap更新に関するステップはすべてスキップされ、この未設定状態単独ではリリースワークフロー全体のステータスに影響しない

#### Scenario: PAT設定済みでtap更新PRの作成が失敗するとリリースワークフローは失敗扱いになる
- **WHEN** `HOMEBREW_TAP_PAT` が設定された状態で、tap更新PRの作成(`brew tap`/`brew trust`/`brew bump-cask-pr`のいずれか)が失敗する
- **THEN** リリースワークフロー全体のステータスは失敗になる

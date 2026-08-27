# homebrew-tap-release-automation

## Purpose

GitHub Releaseの公開をトリガーに、Homebrew tapリポジトリ(`kuchida1981/homebrew-bitwarden-quickaccess`)のCask定義(`version`/`sha256`)を更新するプルリクエストを自動作成し、インストール確認まで行う。手作業でのCask更新(更新忘れ・sha256の転記ミスのリスク)を無くしつつ、tap本体への反映は人間のレビュー・マージを経るようにする。

## Requirements

### Requirement: リリース公開時のHomebrew tap更新PR自動作成
GitHub Releaseが公開され、`.app` のビルド・アップロードが完了した場合、システムはHomebrew tapリポジトリ(`kuchida1981/homebrew-bitwarden-quickaccess`)に対し、新しいバージョン・sha256を反映したCask更新のプルリクエストを自動的に作成しなければならない(SHALL)。tap本体(デフォルトブランチ)への直接反映は行ってはならない(SHALL NOT)。

#### Scenario: リリース公開後にtap更新PRが作成される
- **WHEN** `vX.Y.Z` のGitHub Releaseが公開され、`.app` アセットのアップロードが完了する
- **THEN** tapリポジトリに、`version`が`X.Y.Z`に、`sha256`が新アセットのハッシュ値に更新されたCask変更を含むプルリクエストが作成される

### Requirement: Cask更新のlint
Homebrew tap更新PRを作成する際、システムはCask定義に対する構文・スタイルチェック(`brew audit`/`brew style`相当)を実行しなければならない(SHALL)。

#### Scenario: 不正なCask定義になる変更ではPR作成が失敗する
- **WHEN** Cask定義の更新結果がHomebrewのaudit/styleチェックに違反する
- **THEN** プルリクエストの作成は行われず、リリースワークフロー内でエラーとして記録される

### Requirement: インストール確認(非ブロッキング)
Homebrew tap更新PRの作成後、システムは作成されたPRの内容を用いて `brew install --cask` によるインストールが完了することを確認しなければならない(SHALL)。この確認が失敗しても、リリースワークフロー全体を失敗として扱ってはならない(SHALL NOT)。

#### Scenario: インストール確認が成功する
- **WHEN** tap更新PRのブランチ内容で `brew install --cask` を実行する
- **THEN** インストールが完了し、失敗した場合と異なりワークフローの成否に影響しない

#### Scenario: インストール確認が失敗してもワークフロー自体は成功扱いになる
- **WHEN** `brew install --cask` の実行がエラーで終了する
- **THEN** リリースワークフロー全体のステータスは、この失敗単独では失敗にならない

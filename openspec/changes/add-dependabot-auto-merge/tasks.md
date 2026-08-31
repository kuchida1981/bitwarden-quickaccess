## 1. Dependabot 設定

- [x] 1.1 `.github/dependabot.yml` を新規作成する(`cargo`: `directory: /app/src-tauri`, `github-actions`: `directory: /`、両方 `schedule.interval: weekly`、`commit-message.prefix` を cargo=`chore(deps)` / github-actions=`ci(deps)` に設定)。`python3 -c "import yaml; yaml.safe_load(open('.github/dependabot.yml'))"` で有効なYAMLであることを確認する

## 2. リポジトリ設定変更(要ユーザー確認)

- [x] 2.1 `gh api -X PATCH repos/{owner}/{repo} -f allow_auto_merge=true` でリポジトリの auto-merge 許可を有効化し、`gh api repos/{owner}/{repo} --jq '.allow_auto_merge'` が `true` を返すことを確認する
- [x] 2.2 main の CI 必須化を確認する。classic branch protection API は 404(未保護)を返すが、既存の ruleset `default`(id: 20505186)が `required_status_checks` ルールで `test` を既に必須化していることを `gh api repos/{owner}/{repo}/rulesets/20505186 --jq '.rules[] | select(.type=="required_status_checks")'` で確認した。誤って classic branch protection を追加してしまったが、ruleset と重複・一部矛盾するため削除し、ruleset側の設定のみを正とする(追加のブランチ保護ルールは作成しない)

## 3. 自動マージワークフロー

- [x] 3.1 `.github/workflows/dependabot-auto-merge.yml` を新規作成する(`on: pull_request`、`permissions: {contents: write, pull-requests: write}`、`if: github.actor == 'dependabot[bot]'`、`dependabot/fetch-metadata@v2` で `update-type` を取得し、`semver-major` 以外の場合のみ `gh pr merge --auto --merge` を実行)。`python3 -c "import yaml; yaml.safe_load(open('.github/workflows/dependabot-auto-merge.yml'))"` で有効なYAMLであることを確認する
- [x] 3.2 `actionlint`(利用可能な場合)または `act -n`/目視レビューでワークフロー構文が既存の `ci.yml`/`release.yml` のスタイル(インデント、ステップ命名)と一貫していることを確認する

## 4. ドキュメント更新

- [x] 4.1 CONTRIBUTING.md または README.md に Dependabot 運用(cargo/github-actions週次更新、semver-patch/minorは自動マージ、semver-majorは手動レビュー)についての記載が必要か検討し、必要であれば追記する

## 5. 動作確認

- [ ] 5.1 全ファイルをコミットしPR作成後、`gh pr checks --watch` でCIが通ることを確認する
- [ ] 5.2 PRマージ後、GitHub UI の Insights → Dependency graph → Dependabot から `Check for updates` を実行し、少なくとも1件の更新プルリクエストが実際に作成されることを確認する(スケジュール実行を待たずに動作確認するため)

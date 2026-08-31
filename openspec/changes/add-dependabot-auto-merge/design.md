## Context

現状 `.github/dependabot.yml` は存在せず、main ブランチにも保護ルールが一切設定されていない(`allow_auto_merge` も false)。CI(`ci.yml`)は `pull_request:` トリガーで secrets を使わず build/test/clippy を実行しており、同一リポジトリ内のブランチとして作成される Dependabot PR に対しても追加設定なしで動作する。動機は proposal.md 参照。

## Goals / Non-Goals

**Goals:**
- Dependabot による cargo(`app/src-tauri`)・github-actions(`/`)の週次更新検出
- semver-patch/minor の Dependabot PR を CI 通過後に自動マージ(マージコミット方式)
- semver-major は自動マージ対象から除外し、人間のレビューを必須のまま残す

**Non-Goals:**
- Dependabot 以外の一般PRのマージフローや、OpenSpec change の実装フロー(agy委譲等)を変更すること
- semver-major 更新の自動追従・自動マージ
- npm/yarn等、cargo・github-actions 以外のエコシステムの追加

## Decisions

### 1. ブランチ保護ルールで CI を必須ステータスチェックにする
GitHub の「Auto-merge」は、必須ステータスチェックが設定されていない場合、PRがmergeable判定された時点(≒作成直後)で即マージしてしまう。「CI通過後に自動マージ」を実現するには、Dependabot専用の仕組みではなく **main ブランチ保護ルールで CI(`test`ジョブ)を必須化する** ことが前提になる。
- 代替案: ワークフロー側で `workflow_run` イベントを使い、CI完了後に自身でマージ判定するポーリング/イベント駆動の仕組みを自作する方法もあるが、GitHub標準の auto-merge + 必須ステータスチェックの組み合わせで完結でき、実装・保守コストが低いためこちらを採用する。
- 副作用として、Dependabot以外の通常PRも今後は CI 通過がマージの前提になる(scope拡大、ユーザー合意済み)。

### 2. `dependabot/fetch-metadata` で update-type を判定し `gh pr merge --auto` で有効化
新規ワークフロー `.github/workflows/dependabot-auto-merge.yml` を追加する。
- トリガー: `pull_request`(同一リポジトリ内のブランチのため `pull_request_target` は不要。secrets を使わないため安全側で通常の `pull_request` を採用)
- `if: github.actor == 'dependabot[bot]'` でDependabot PRのみに限定
- `dependabot/fetch-metadata@v2` で `update-type` (`version-update:semver-patch` / `semver-minor` / `semver-major`) を取得
- `update-type` が `semver-major` 以外の場合のみ `gh pr merge --auto --merge "$PR_URL"` を実行し、GitHub標準の auto-merge を有効化する(実際のマージはブランチ保護のCI必須チェックが成功した時点でGitHubが行う)
- マージ方式は既存のPR履歴(マージコミット)に揃え `--merge` を使う(squash/rebaseにしない)

### 3. 0.x系クレート(pre-1.0)の扱い
`reqwest = "0.12"` 等の0.x系クレートは SemVer上 minor バンプが破壊的変更を含みうるが、他のminor/patch更新と区別せず自動マージ対象に含める(ユーザー合意済み)。安全網はCI(`cargo build` / `cargo test` / `cargo clippy -D warnings`)に委ねる。将来的に実際の破壊が頻発するようであれば、対象クレートを ignore ルールで除外する運用変更を検討する。

### 4. commit-message prefix
Dependabot の生成するコミットメッセージは、既存のconventional prefix運用(`feat:` `fix:` `ci:` `chore:` `docs(openspec):`)に寄せ、`.github/dependabot.yml` の `commit-message.prefix` で cargo エコシステムは `chore(deps)`、github-actions エコシステムは `ci(deps)` を指定する。

### 5. issue #89 との関係
`tauri-apps/tauri-action@v0` の更新(issue #89)は個別対応せず、github-actions エコシステムを有効化した Dependabot が自動的に更新PRを提起する想定とする。そのPRがマージされた時点で #89 は実質的に解決される。本changeのタスクでは #89 を明示的にクローズする作業は含めない(Dependabot PRのマージ後に確認・クローズする運用)。

## Risks / Trade-offs

- [Risk] 0.x系クレートのminor更新が実際に破壊的変更を含み、CIのカバレッジ外の挙動変化を自動マージしてしまう → Mitigation: CI(build/test/clippy)を安全網とし、問題が顕在化したら該当クレートをignore対象に追加する運用変更で対応する
- [Risk] ブランチ保護ルールの追加により、Dependabot以外の通常PRもCI通過が必須になり、これまで可能だった「CI未確認でのマージ」ができなくなる → Mitigation: 既にCLAUDE.mdの運用で `gh pr checks --watch` を必須としており、実質的な運用変更は小さい
- [Risk] `dependabot/fetch-metadata` のメジャーバージョン(`@v2`)が将来的に破壊的変更を含む可能性 → Mitigation: 本体のDependabot設定自身がこのアクションの更新も検出するため、更新PRとして通常のレビューフローに乗る

## Migration Plan

- 新規ファイル追加とリポジトリ設定変更のみで、既存コードへの影響はない
- ロールバックは `.github/dependabot.yml` と `.github/workflows/dependabot-auto-merge.yml` の削除、ブランチ保護ルールの解除、`allow_auto_merge` の無効化で即座に可能

## Why

`.github/dependabot.yml` が存在せず、Cargo 依存関係(`thiserror` v1→v2 など)と GitHub Actions(`tauri-apps/tauri-action@v0` など、issue #89)のバージョン更新が自動化されていない。手動での追跡は漏れやすく、更新PRの受け取り自体が発生していないため脆弱性・不具合修正の反映も遅れる。issue #121 の対応として Dependabot を導入し、あわせて minor/patch 更新は CI 通過後に自動マージできるようにする。

## What Changes

- `.github/dependabot.yml` を新規作成し、`cargo`(`app/src-tauri`)と `github-actions`(`/`)の週次更新を有効化する
- リポジトリ設定 `allow_auto_merge` を有効化する(main は既存の ruleset `default` で CI(`test`)が既に必須ステータスチェックとして設定済みのため、追加のブランチ保護は不要)
- `.github/workflows/dependabot-auto-merge.yml` を新規作成し、Dependabot PR のうち `semver-patch` / `semver-minor` の更新を CI 通過後にマージコミット方式で自動マージする(`semver-major` は対象外、手動レビューに残す)

## Capabilities

### New Capabilities
- `dependency-update-automation`: Dependabot による依存関係更新の自動検出(cargo / github-actions)と、CI通過を前提とした minor/patch 更新の自動マージポリシー

### Modified Capabilities
(なし)

## Impact

- 影響ファイル: `.github/dependabot.yml`(新規)、`.github/workflows/dependabot-auto-merge.yml`(新規)
- 影響設定: リポジトリの `allow_auto_merge` 設定(main の CI 必須化は既存の ruleset で対応済みのため変更なし)
- 関連issue: #121(本体)、#89(tauri-action更新はDependabotのPRとして自動的に提起される見込み)
- 依存関係・外部サービスへの影響なし(GitHub標準機能のみ使用)

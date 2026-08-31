## Why

`release.yml` の Homebrew tap更新ステップ群はすべて `continue-on-error: true` になっており、`HOMEBREW_TAP_PAT` 未設定という限定的なケースだけでなく、あらゆる実際の不具合も無条件に握りつぶしてリリースジョブを成功表示にしてしまう。v1.5.0の実リリースで、tauri-action v1へのアップグレードによりリリースアセットの命名規則が変わり、tap側の `url` テンプレートが古いままだったため `brew bump-cask-pr` が404で失敗したが、この失敗は誰にも気づかれずワークフローは成功表示のままだった(tapは実際には更新されていなかった)。

## What Changes

- `.github/workflows/release.yml` の Homebrew tap更新ステップの `if`/`continue-on-error` を見直す:
  - `HOMEBREW_TAP_PAT` が未設定の場合のみ、tap更新ステップ一式(`Tap the Homebrew repository` / `Trust the Homebrew tap` / `Open a Homebrew tap update PR`)をスキップし、リリースジョブは成功のままにする
  - `HOMEBREW_TAP_PAT` が設定されているにも関わらずこれらのステップが失敗した場合は、`continue-on-error` を外し、リリースジョブ全体を失敗表示にする
  - 後続の「インストール確認」系ステップ(`Resolve the tap PR branch` / `Verify the cask installs`)は、既存spec要件(インストール確認は非ブロッキング)により `continue-on-error: true` を維持する
- CONTRIBUTING.md に、この挙動変更と v1.5.0 での実例を追記する

## Capabilities

### New Capabilities
(なし)

### Modified Capabilities
- `homebrew-tap-release-automation`: PAT未設定時のスキップと、PR作成失敗時のジョブ失敗化という、これまで未規定だった挙動を新しい要件として追加する

## Impact

- 影響ファイル: `.github/workflows/release.yml`、`CONTRIBUTING.md`
- 影響しないもの: `Resolve the tap PR branch` / `Verify the cask installs` の非ブロッキング挙動(既存要件のまま変更なし)
- 関連: v1.5.0リリースでの実インシデント(tap更新PR未作成に気づけなかった問題)

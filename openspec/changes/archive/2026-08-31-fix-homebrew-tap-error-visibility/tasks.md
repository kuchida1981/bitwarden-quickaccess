## 1. release.yml の修正

- [x] 1.1 job レベルの `env` に `HOMEBREW_TAP_PAT_CONFIGURED: ${{ secrets.HOMEBREW_TAP_PAT != '' }}` を追加する。`python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml'))"` で有効なYAMLであることを確認する
- [x] 1.2 `Tap the Homebrew repository` / `Trust the Homebrew tap` / `Open a Homebrew tap update PR` の3ステップに `if: ${{ env.HOMEBREW_TAP_PAT_CONFIGURED == 'true' }}` を追加し、`continue-on-error: true` を削除する
- [x] 1.3 `Resolve the tap PR branch` / `Verify the cask installs` は既存の `if:` 条件を維持したまま `continue-on-error: true` を残す(既存spec要件により非ブロッキングのまま)
- [x] 1.4 `actionlint .github/workflows/release.yml` で構文・コンテキスト参照の妥当性を確認する(`secrets` コンテキストをステップの`if:`で直接参照していないこと)

## 2. ドキュメント更新

- [x] 2.1 CONTRIBUTING.md に、この挙動変更とv1.5.0での実インシデントを追記する

## 3. Spec同期

- [x] 3.1 `openspec/changes/fix-homebrew-tap-error-visibility/specs/homebrew-tap-release-automation/spec.md` に ADDED Requirement として新要件を記述する(archive時にmain specへ同期する)

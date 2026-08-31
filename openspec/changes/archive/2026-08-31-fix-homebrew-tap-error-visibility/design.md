## Context

`release.yml` の Homebrew tap更新ステップ群には元々、全ステップ一律で `continue-on-error: true` が付いていた。意図はコメントに記載の通り「PAT未登録等の理由で失敗しても、その一事だけでリリースジョブ全体を失敗表示にしたくない」だったが、実装はこの意図より広く、あらゆる失敗原因(URL 404等)を無条件に握りつぶしていた。v1.5.0リリースで実際にこの問題が発生した(詳細はproposal.md参照)。

## Goals / Non-Goals

**Goals:**
- `HOMEBREW_TAP_PAT` 未設定という既知・許容されたケースのみを非ブロッキングにする
- それ以外のtap更新PR作成失敗は、リリースジョブを失敗表示にして気づけるようにする
- 既存spec要件(インストール確認ステップの非ブロッキング)は変更しない

**Non-Goals:**
- `brew bump-cask-pr` の再実行時の冪等性(重複ブランチ/PRでの失敗)を解消すること。既存の問題であり、対処はCONTRIBUTING.mdの手動フォールバック手順に委ねる
- Homebrew tap自動化の設計全体(issue #77 で導入された仕組み)を見直すこと

## Decisions

### 1. `HOMEBREW_TAP_PAT` の有無を job レベルの `env` で判定する
ステップの `if:` 条件式では `secrets` コンテキストを直接参照できない(GitHub Actionsの制約、`actionlint` で検出)。そのため job レベルの `env` に `HOMEBREW_TAP_PAT_CONFIGURED: ${{ secrets.HOMEBREW_TAP_PAT != '' }}` を定義し、各ステップの `if:` はこの `env` 値を参照する。

### 2. `continue-on-error` を外す範囲を「PR作成」ステップに限定する
`Tap the Homebrew repository` / `Trust the Homebrew tap` / `Open a Homebrew tap update PR` の3ステップから `continue-on-error: true` を外す(PAT未設定時は `if:` でスキップされるため、実行される場合の失敗は実際の不具合とみなす)。
一方 `Resolve the tap PR branch` / `Verify the cask installs` は、既存spec要件「インストール確認(非ブロッキング)」により失敗してもジョブを失敗にしてはならない(SHALL NOT)ため、`continue-on-error: true` を維持する。

## Risks / Trade-offs

- [Risk] リリースワークフローを「Re-run jobs」で再実行すると、前回実行で作成済みのtap側ブランチ/PRが残っている場合 `brew bump-cask-pr` が重複で失敗し、ジョブが失敗表示になる(コードレビューで指摘) → Mitigation: これは元々存在した問題(以前はcontinue-on-errorで無条件に握りつぶされていただけ)。対処は他の失敗原因と同じくCONTRIBUTING.mdの手動フォールバック手順(該当ブランチ/PRを削除してから再実行)に従う。冪等性の作り込みは本changeのスコープ外とする
- [Risk] `HOMEBREW_TAP_PAT` が設定されているが無効(失効・権限不足)な場合も「実際の不具合」としてジョブが失敗表示になる → Mitigation: これは意図した挙動(PATが無効なのは是正すべき設定ミスであり、隠すべきではない)

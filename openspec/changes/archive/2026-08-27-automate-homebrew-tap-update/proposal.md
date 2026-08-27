## Why

GitHub issue #77: `gh release create` で公式リリースを公開すると `.github/workflows/release.yml` が `.app` のビルド・アップロードまでは自動化しているが、その後の Homebrew tap(`kuchida1981/homebrew-bitwarden-quickaccess`)のCask更新(`version`/`sha256`の書き換え、lint、コミット・プッシュ)は`CONTRIBUTING.md`記載の手順に従った手作業のままである。手作業である以上、更新忘れ・sha256の転記ミスといったリスクが残る。

## What Changes

- `.github/workflows/release.yml` に、`.app` のビルド・アップロード完了後のステップとして、Homebrew公式CLIコマンド `brew bump-cask-pr` によるtap更新自動化を追加する。このコマンドは、バージョン/sha256の更新・`brew audit`/`brew style --fix`の実行・ブランチ作成・コミット・プッシュ・PR作成までを一括で行う(このため「直接pushかPR作成か」という論点は、ツールの標準動作であるPR作成方式を採用することで解決する)。
- PR作成後、非ブロッキング(失敗してもワークフロー自体は成功扱い)のステップとして、作成されたPRのブランチ内容で実際に `brew install --cask` を実行し、インストールが完了することを確認する(GUI起動確認は対象外)。
- クロスリポジトリへのpush・PR作成のため、tapリポジトリへの書き込み権限を持つ個人アクセストークン(PAT)を、このリポジトリのActions Secretとして**ユーザー自身が事前に登録する必要がある**(Claude Codeでは代行不可)。
- `CONTRIBUTING.md` のリリース手順から、自動化された「3. Homebrew tapのCaskを更新する」ステップを、確認・トラブルシューティング手順に更新する。

## Capabilities

### New Capabilities
- `homebrew-tap-release-automation`: リリース公開をトリガーに、Homebrew tapへのCask更新PRを自動作成し、インストール確認まで行う機能。

### Modified Capabilities
(なし。`homebrew-distribution`はREADMEのユーザー向け手順に関する既存要件であり、本changeが自動化するのはメンテナー向けのリリース内部プロセスのため、別capabilityとして切り出す)

## Impact

- `.github/workflows/release.yml`: Homebrew tap更新ステップの追加
- `CONTRIBUTING.md`: リリース手順の更新(自動化後の確認手順に変更)
- 破壊的変更なし。既存の手動手順は、自動化が何らかの理由で失敗した場合のフォールバックとして`CONTRIBUTING.md`に残す。

## ユーザーが事前に行う必要がある作業(重要)

以下はClaude Codeでは代行できず、リリース自動化が実際に動作する前に**あなた自身**が対応する必要があります:

1. tapリポジトリ(`kuchida1981/homebrew-bitwarden-quickaccess`)への書き込み権限を持つ個人アクセストークン(fine-grained PAT推奨、対象リポジトリをtapに限定し、Contents: Read and write / Pull requests: Read and write の権限を付与)を発行する。
2. 本リポジトリ(`kuchida1981/bitwarden-quickaccess`)の Settings > Secrets and variables > Actions に、上記PATを `HOMEBREW_TAP_PAT` という名前でSecretとして登録する。

この登録が完了するまで、実装されたワークフローは(PATが無いため)tap更新ステップで失敗します。次回の実リリースで初めて実地検証されることになります。

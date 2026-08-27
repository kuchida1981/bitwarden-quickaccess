## Context

現在の手動手順(`CONTRIBUTING.md`)は、tapリポジトリをローカルにcloneし、`gh release view` でタグ・sha256(digest)を取得し、`Casks/bw-quickaccess.rb` の `version`/`sha256` を手で書き換え、`brew style --cask`/`brew audit --cask` でlintし、`brew reinstall --cask` で実地確認し、コミット・プッシュする、という流れ。

調査の結果、issueが提案していた「Homebrew公式寄りのアクション」は実際にはGitHub Actionではなく、Homebrew本体に組み込まれている **`brew bump-cask-pr`** というCLIサブコマンドだった。このコマンドは以下を1コマンドで行う:
- Caskの `url` テンプレート(`#{version}` を含む)に新バージョンを当てはめてダウンロードし、sha256を自動算出(`--sha256`で明示指定も可能)
- Cask定義ファイルの `version`/`sha256` を書き換え
- `brew audit`/`brew style --fix` を実行(`--no-audit`/`--no-style`で無効化可能)
- 新しいブランチを作成し、コミット・プッシュ
- 対象リポジトリへのPRを作成(`--no-browse`でブラウザを開かずURLを標準出力、`--no-fork`で自分のリポジトリへの直接ブランチ作成/PRにする)

macOSランナー(`macos-latest`)にはHomebrewがプリインストールされているため、追加のセットアップ(`Homebrew/actions/setup-homebrew`等)は不要。

## Goals / Non-Goals

**Goals:**
- リリース公開(`gh release create`)をトリガーに、Homebrew tapへのCask更新PRを自動作成する。
- Cask更新の際に`brew audit`/`brew style`によるlintを自動で行う(ツールの標準動作)。
- 作成したPRの内容で実際に `brew install --cask` が完了することを、リリースワークフロー内で確認する(非ブロッキング)。

**Non-Goals:**
- 人間の承認を経ないtap本体(mainブランチ)への直接反映(完全自動デプロイ)。PRはあくまで人間がマージするまでtap本体に影響しない。
- GUIアプリの起動確認の自動化(Homebrewのインストール完了確認までに留める)。
- tapリポジトリ側(`homebrew-bitwarden-quickaccess`)のワークフロー変更(本リポジトリ側の変更のみで完結させる)。

## Decisions

- **`brew bump-cask-pr` を採用し、自前のsedスクリプトは書かない**。バージョン/sha256更新・lint・PR作成が1コマンドに統合されており、Cask DSLとしての妥当性もHomebrew本体のコードでチェックされるため、自前実装よりフォーマットミスのリスクが低い。
- **`--no-fork` を指定する**。tapリポジトリへの書き込み権限を持つPATを使う前提のため、フォークは不要で、同一リポジトリ内にブランチを作成しPRを開く。
- **sha256は明示指定せず、コマンドの自動ダウンロード・算出に任せる**。これにより「新しいURLが実際にダウンロード可能である」ことの検証も同時に行われる(明示指定するとこの検証を省略してしまう)。
- **`HOMEBREW_GITHUB_API_TOKEN` 環境変数にPATを渡す**。`brew`本体がGitHub API呼び出し(ブランチ作成・PR作成)に使う標準の環境変数であり、追加のツール(`gh` CLI等)を経由させる必要がない。
- **リリースタグから`v`を除いたバージョン文字列は「Sync Cargo.toml version」ステップでのみ算出し、`id`経由の出力(`steps.release_version.outputs.version`)として後続ステップに渡す**(コードレビューで発覚した重複への対応)。同じ導出ロジック(`${RELEASE_TAG#v}`)を複数箇所に重複させると、将来pre-releaseサフィックス対応等でロジックを変更する際に片方だけ更新し忘れるリスクがある。
- **インストール確認は、`brew bump-cask-pr` が作成したPRの実際のブランチ内容に対して行う**。`brew bump-cask-pr` 実行後のローカルtapのgit状態(コミット後にブランチを切り替えるかどうか)はHomebrew内部実装に依存し確実ではないため、明示的に `git checkout` してから `brew install --cask` を実行する。
- **対象PRのブランチ名は `gh pr list` でtapリポジトリの最新openPRを問い合わせて取得する**(当初は `brew bump-cask-pr` の標準出力からPR URLを正規表現で抽出する方式だったが、コードレビューで2つの問題が判明し撤回した: (1) GitHub Actionsのbashはデフォルトで`set -e -o pipefail`のため、`grep`が無マッチで失敗するとその場でスクリプトが終了し、想定していたカスタム診断メッセージに到達しない、(2) brewの自由形式な標準出力を正規表現で拾う方式自体が、想定外の`pull/N`文字列(無関係なPRへの言及等)を誤って拾うリスクを持つ)。`gh pr list --state open --json headRefName,createdAt --jq 'sort_by(.createdAt) | last | .headRefName // empty'` は、brewの出力形式に一切依存せず、該当PRが無い場合も空文字列を返して正常終了する(ローカルで`jq`の挙動を検証済み)ため、後続の空チェックが確実に機能する。tapに古い未マージPRが残っている状態で新しいPRが作成された場合でも、`createdAt`でソートして最新を選ぶため正しく新しい方を拾う。

**追記(コードレビューで発覚・再修正)**: 上記だけでは「今回の`brew bump-cask-pr`自体が失敗し(PAT不正等)、かつtapに無関係な古いPRが残っている」場合に、その無関係なPRを誤って拾い、インストール確認が偽陽性の成功を報告してしまう(=本当の失敗を覆い隠す)リスクが残っていた。これを防ぐため2点追加した: (1) `Open a Homebrew tap update PR` ステップに `id: bump_cask_pr` を付け、後続の `Resolve the tap PR branch`/`Verify the cask installs` を `if: steps.bump_cask_pr.outcome == 'success'`(および前段成功時のみ)の条件付きにし、PR作成自体が失敗した場合は後続を実行しない。(2) `gh pr list` のjqフィルタに `select(.headRefName | startswith("bump-bw-quickaccess"))` を追加し、無関係なPR(人間による手動PR・Dependabot等)を最初から除外する。ブランチ名の命名規則(`bump-<cask token>...`)はHomebrew本体のソースコード(`dev-cmd/bump-cask-pr.rb`)で確認済み。
- **インストール確認は non-blocking にする**(`continue-on-error: true` 等)。PR自体は既に作成済みであり、確認ステップの失敗でジョブ全体を失敗させると「PRは出来ているのにワークフローは失敗表示」という混乱を招くため、警告のみに留める。
- **tap更新関連のステップ(tap登録・PR作成・ブランチ解決・インストール確認)は全て non-blocking にする**(実装時に判断を拡大)。当初はインストール確認ステップのみを非ブロッキングにする想定だったが、「Open a Homebrew tap update PR」自体もPAT未登録時には確実に失敗するため、そこだけブロッキングのままだと、既に完了しているアプリ本体のビルド・アップロードとは無関係に、tap更新の失敗だけでリリースジョブ全体が失敗表示になってしまう。`continue-on-error: true` の各ステップは失敗してもUI上は警告として可視化される(完全に握りつぶされるわけではない)ため、可視性は保ちつつジョブ全体の成否には影響しない設計にした。

## Risks / Trade-offs

- [PATの権限が広すぎる/狭すぎる] → fine-grained PATを使い、対象リポジトリをtapリポジトリ1つに限定し、必要な権限(Contents, Pull requests の読み書き)のみ付与するよう`CONTRIBUTING.md`/proposal.mdで明記する。
- [`brew bump-cask-pr` の正確な内部挙動(ローカルgit状態の扱い等)は実機のGitHub Actions環境で経験的に検証したものではなく、`--help`出力とHomebrew本体の一般的な設計からの推測を含む] → 次回の実リリースが最初の実地検証になる。失敗した場合は`CONTRIBUTING.md`記載の手動手順にフォールバックできるようにしておく(本changeでは削除しない)。
- [PAT未登録の状態でこの変更をマージすると、次回リリース時にtap更新ステップが失敗する] → ワークフロー全体を失敗させず、後続の通常のアプリビルド・アップロード自体は完了させる設計にする(tap更新は最後のステップ群として追加し、それより前のステップの成否には影響しない)。

## 1. リリースワークフローへのHomebrew tap更新自動化の追加

- [x] 1.1 `.github/workflows/release.yml` の「Build and upload the macOS app to the release」ステップの後に、新しいステップ「Tap the Homebrew repository」を追加する: `brew tap kuchida1981/bitwarden-quickaccess` を実行する。
- [x] 1.2 続けて「Open a Homebrew tap update PR」ステップを追加する。`HOMEBREW_GITHUB_API_TOKEN: ${{ secrets.HOMEBREW_TAP_PAT }}` を環境変数に設定し、以下を実行する:
  ```bash
  version="${RELEASE_TAG#v}"
  brew bump-cask-pr --no-fork --no-browse --version="$version" bw-quickaccess | tee bump-cask-pr-output.txt
  ```
  (`RELEASE_TAG` は `${{ github.event.release.tag_name }}` を別途envで渡す。出力は次のステップでPR番号/URLを取得するためファイルに保存する。)
- [x] 1.3 上記ステップの出力からPRのURL(`https://github.com/kuchida1981/homebrew-bitwarden-quickaccess/pull/<番号>` 形式)を抽出し、`gh pr view <番号> --repo kuchida1981/homebrew-bitwarden-quickaccess --json headRefName -q .headRefName` でブランチ名を取得するステップ「Resolve the tap PR branch」を追加する(`GH_TOKEN` にはtap用PATを使う必要がある点に注意: `GH_TOKEN: ${{ secrets.HOMEBREW_TAP_PAT }}`)。
- [x] 1.4 「Verify the cask installs (non-blocking)」ステップを追加する。取得したブランチ名で `$(brew --repository)/Library/Taps/kuchida1981/homebrew-bitwarden-quickaccess` を `git checkout` した上で `brew install --cask --no-quarantine bw-quickaccess` を実行する。ステップ全体に `continue-on-error: true` を設定し、失敗してもジョブ全体を失敗させないようにする。(ローカルブランチが無い場合のfetchフォールバックも実装)

**追記(実装時に判断を拡大)**: 1.1〜1.3のステップにも `continue-on-error: true` を追加した。PAT未登録時は1.2が確実に失敗するが、その時点でアプリ本体のビルド・アップロードは既に完了しているため、tap更新の失敗だけでリリースジョブ全体を失敗表示にしないようにするため(design.md参照)。

## 2. ドキュメント更新

- [x] 2.1 `CONTRIBUTING.md` の「### 3. Homebrew tapのCaskを更新する」セクションを、「自動化されたPRが作成されるので、その内容を確認してマージする」という趣旨に更新する。既存の手動手順(sha256取得・Cask書き換え・lint・実機確認・コミット/プッシュ)は、自動化が失敗した場合のトラブルシューティング手順として残す。
- [x] 2.2 `CONTRIBUTING.md` に、`HOMEBREW_TAP_PAT` シークレットのセットアップ手順(PATのスコープ・登録場所)を明記するセクションを追加する。

## 3. 動作確認

- [x] 3.1 `.github/workflows/release.yml` の YAML構文が正しいことを確認する(`yamllint` 等が使えればそれで、無ければ手動レビュー)。(2026-08-27 `python3 -m yaml` および `actionlint` で検証、いずれも警告なし)
- [x] 3.2 `brew bump-cask-pr --help` の出力と実装内容を突き合わせ、フラグの使い方に誤りが無いか再確認する。(design.md作成時に実行済み。`--no-fork`/`--no-browse`/`--version`いずれも実際のヘルプ出力に存在することを確認済み)
- [ ] 3.3 ユーザーに対し、`HOMEBREW_TAP_PAT` シークレットの登録を依頼する(このタスクはユーザー自身の対応が必要であり、登録完了をもって完了とする)。
- [ ] 3.4 実際のリリース(次回のバージョンアップ時)で、tap更新PRが正しく作成されることを確認する。**このタスクは本change内では検証できず、次回リリース時に別途確認する。**

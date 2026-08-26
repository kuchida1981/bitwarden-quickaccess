# Contributing

開発環境のセットアップ・セルフビルド手順は [README.md](README.md#option-3-self-build-from-source) を参照してください。AIエージェント(Claude Code / Antigravity CLI)を使った開発ワークフロー(OpenSpecでの提案・実装フロー、ツールの役割分担)については [CLAUDE.md](CLAUDE.md) を参照してください。

## Releasing

新しいバージョンをリリースするときの手順。以下の順序で実行する。

### 1. GitHub Releaseを作成する

`vX.Y.Z` 形式のタグ(セマンティックバージョニング)でGitHub Releaseを作成する。

```bash
gh release create vX.Y.Z --title "vX.Y.Z" --notes "..."
```

`Cargo.toml` の `version` を事前に手動で書き換える必要はない。`.github/workflows/release.yml` がビルド時にリリースタグから自動で `Cargo.toml` の `version`(アプリバンドルのメタデータ用)を同期する(「Sync Cargo.toml version with the release tag」ステップ)。この同期はビルド用のチェックアウト内でのみ行われ、mainブランチへのコミットは発生しない(ブランチ保護によりCIから直接pushできないため)。

これとは別に、トレイメニュー内に表示されるバージョン文字列はビルド時の `git describe` から動的に導出される(セルフビルドでも正確な表示になる)。この2つは別々の経路で決まる値であり、`Cargo.toml` の同期を省略してよいわけではない点に注意する。

リリースノートは過去のリリース(`gh release view v1.1.0 --json body` 等で参照可能)の構成に合わせる: 新機能・バグ修正・インストール方法・対象OSのセクションに分ける。

### 2. ビルド完了を待つ

Releaseの公開をトリガーに `.github/workflows/release.yml` が起動し、`.app` をビルドしてリリースアセット(`bw-quickaccess_aarch64.app.tar.gz`)をアップロードする。

```bash
gh run list --workflow=release.yml --limit 1
gh run watch <run-id> --exit-status
```

### 3. Homebrew tapのCaskを更新する

tapリポジトリ: https://github.com/kuchida1981/homebrew-bitwarden-quickaccess (別リポジトリ。ローカルにcloneして作業する)

1. 新しいリリースのタグとアセットのsha256を取得する(ダウンロードして自分で計算する必要はない。`digest` フィールドに含まれている):
   ```bash
   gh release view vX.Y.Z --repo kuchida1981/bitwarden-quickaccess --json tagName,assets
   ```
2. tapリポジトリの `Casks/bw-quickaccess.rb` の `version` と `sha256` を、1で取得した値に更新する。
3. ローカルで `brew tap kuchida1981/bitwarden-quickaccess` 済みであれば、変更をtap先のディレクトリ(`$(brew --repository)/Library/Taps/kuchida1981/homebrew-bitwarden-quickaccess/Casks/bw-quickaccess.rb`)にも反映してlintする:
   ```bash
   brew style --cask bw-quickaccess
   brew audit --cask bw-quickaccess
   ```
4. `brew reinstall --cask bw-quickaccess` で実際にインストールし直し、起動・アンロックできることを実機確認する。
5. tapリポジトリの変更をコミット・プッシュする。

tapリポジトリ自身のREADMEにも同じ手順を記載している。

### 4. マイルストーンをクローズする(該当する場合)

そのリリースが特定のGitHub Milestoneに対応する場合、完了後にクローズする:

```bash
gh api repos/kuchida1981/bitwarden-quickaccess/milestones/<番号> -X PATCH -f state=closed
```

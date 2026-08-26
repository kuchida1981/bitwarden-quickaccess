## Why

現在のインストール手段はGitHub Releasesからの手動ダウンロードかセルフビルドのみで、Homebrewでのインストールに対応していない(issue #49)。より手軽なインストール手段として、`brew install --cask` に対応したい。

## What Changes

- Homebrew tap用の新規GitHubリポジトリ `kuchida1981/homebrew-bitwarden-quickaccess` を作成する(**新規リポジトリの作成を伴う。実行前に確認する**)
- そのリポジトリに、本アプリをインストールするCask定義(`Casks/bw-quickaccess.rb`)を追加する
  - `bw-quickaccess_aarch64.app.tar.gz`(Apple Silicon専用、現行のリリースアセットと同じ)をダウンロードしてインストールする
  - Intel Mac(`x86_64`)では明示的にサポート対象外であることを示す(`depends_on arch: :arm64`)
  - 無署名・非notarizedであることのcaveats(初回起動時のGatekeeper対応方法)を表示する
- README.md / README.ja.md の Install セクションに、Homebrewでのインストール手順を追記する
- 新しいバージョンをリリースするたびにCaskの `version`/`sha256` を更新する手順をドキュメント化する(今回は自動化しない。手動更新の手順書を残す)

## Capabilities

### New Capabilities
- `homebrew-distribution`: Homebrew Cask経由でのインストール手順をREADMEに提供する(tap/Cask自体は別リポジトリの成果物のため、このリポジトリ内で検証可能な要件はドキュメント面に限定される)

### Modified Capabilities
(なし)

## Impact

- 影響コード: なし(Rust/JSの実装変更はない)
- 影響ドキュメント: `README.md`, `README.ja.md`(Installセクション、Homebrewの手順追加)
- 新規リポジトリ: `kuchida1981/homebrew-bitwarden-quickaccess`(tap)。**GitHub上に新しい公開リポジトリを作成する操作を伴うため、実行前にユーザーの明示的な確認を取る**
- 新規外部依存: なし(利用者側は `brew` コマンドのみ)
- 対象外:
  - リリースごとのCask自動更新(GitHub Actions等での自動化)。今回は手動更新の手順のみ整備する
  - Intel Mac向けビルド・配布
  - コード署名・notarization(既存のOut of scopeを踏襲)

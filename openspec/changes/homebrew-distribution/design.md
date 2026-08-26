## Context

現在の配布手段(README.md `## Install`)は以下の2つ。

1. GitHub Releasesから `bw-quickaccess_aarch64.app.tar.gz` を手動ダウンロード
2. `cargo tauri build` によるセルフビルド

最新リリースは `v1.0.0`(タグ)で、リリースアセットの実際のsha256は `6dd69706f9c1032b98482d296fc6ad169d5bebaf39e762ba40dbd64b8bb2c77e`(`gh release view --json assets` の `digest` フィールドから直接取得可能。ダウンロードして自分で計算する必要はない)。

なお `Cargo.toml` の `version` は `0.1.0` のままで、実際に公開されているリリースタグ(`v1.0.0`)と乖離している。トレイメニューのバージョン表示(`about-and-branding` で実装)はCargo.tomlの値を参照するため、この乖離は既存の別問題として認識しているが、本changeのスコープには含めない(Cask側は実際のリリースタグの値を使うため、この乖離があってもCaskの動作自体には影響しない)。

Homebrew Caskは、`brew tap <user>/<repo-suffix>` で `https://github.com/<user>/homebrew-<repo-suffix>` という命名規則のリポジトリを参照する。issue #49で要望されていた `brew tap kuchida1981/bitwarden-quickaccess` というコマンドを成立させるには、`kuchida1981/homebrew-bitwarden-quickaccess` という名前でリポジトリを作成する必要がある。

## Goals / Non-Goals

**Goals:**
- `brew tap kuchida1981/bitwarden-quickaccess && brew install --cask bw-quickaccess` でインストールできるようにする
- 無署名・非notarizedアプリであることに起因する制約(Gatekeeper)をCaskのcaveatsとREADMEで明示する
- 新しいリリースが出た際にCaskを更新する手順を明文化する

**Non-Goals:**
- Cask更新の自動化(GitHub Actions等)。リリース頻度が低いため、今回は手動更新の手順書で足りると判断する
- Intel Mac向けのビルド・配布
- `brew audit --cask` / `brew style --cask` をCIに組み込むこと(tap側のCIは今回のスコープ外。ローカルでのlint確認のみ行う)

## Decisions

### 1. tapリポジトリ名は `kuchida1981/homebrew-bitwarden-quickaccess`

Homebrewの命名規則(`homebrew-<tap名>`)に従う。`brew tap kuchida1981/bitwarden-quickaccess` というissue #49で要望されたコマンドがそのまま使えるようにするため、`<tap名>` 部分は本体リポジトリ名と同じ `bitwarden-quickaccess` にする。

**新規リポジトリの作成が必要**。GitHub上に公開リポジトリを新設する操作であり、実行前にユーザーへ確認する。

### 2. Caskの配置は `Casks/bw-quickaccess.rb`(モダンなtap構成)

近年のHomebrewの推奨構成(`brew tap-new` が生成する構成)に合わせ、リポジトリ直下ではなく `Casks/` サブディレクトリにCaskファイルを置く。

### 3. Cask名は `bw-quickaccess`(Cargo.toml のパッケージ名 / `tauri.conf.json` の `productName` と一致)

issue #49で要望されていた `brew install --cask bw-quickaccess` がそのまま使えるようにする。

### 4. Intel Macは `depends_on arch: :arm64` で明示的にブロックする

現在のリリースアセットはApple Silicon(aarch64)専用で、Intel Mac向けビルドは提供していない(README「Out of scope」参照)。`depends_on arch: :arm64` を指定することで、Intel Macでの `brew install` 時に「このCaskはarm64が必要です」という分かりやすいエラーになり、動かないバイナリを誤ってダウンロードさせずに済む。

### 5. 無署名・非notarizedであることをCaskのcaveatsで案内する

Homebrew Caskは、対象アプリが署名・notarizedされていない場合でも、原則としてGatekeeperの検疫属性(quarantine)を自動では除去しない(古いバージョンのHomebrew Caskには自動除去の挙動があったが、セキュリティ上の理由で現在は行われない)。そのため `brew install --cask bw-quickaccess` でインストールしても、初回起動時にGatekeeperの警告が出ることを想定し、Caskの `caveats` ブロックで以下を案内する。

- 対処法1: Finderで `bw-quickaccess.app` を右クリック(Controlキー+クリック)→「開く」を選択し、ダイアログで「開く」を確認する(README記載の既存の手順と同じ)
- 対処法2: `brew install --cask --no-quarantine bw-quickaccess`(検疫属性を最初から付与しないインストールオプション)を使う

**代替案: Caskの `zap`/`postflight` で検疫属性を自動除去するスクリプトを仕込む** — 却下。ユーザーの許可なく検疫属性を除去するスクリプトをCask側に仕込むのは、たとえ自分のアプリのためであってもセキュリティ上望ましい手法ではなく、Homebrew Cask公式のガイドラインにも反する。ユーザーが `--no-quarantine` フラグを能動的に選ぶ形にする。

### 6. Cask更新は手動、手順をコミットメッセージ規約的なチェックリストとしてtapリポジトリのREADMEに残す

新しいバージョンをリリースするたびに、以下の手順を実行する:

1. `gh release view --json tagName,assets` で最新タグとアセットの `digest`(sha256)を取得する
2. `Casks/bw-quickaccess.rb` の `version` と `sha256` を更新する
3. `brew audit --cask bw-quickaccess` / `brew style --cask bw-quickaccess` でlintする
4. tapリポジトリにコミット・プッシュする

この手順をtapリポジトリの README に明文化する。自動化(GitHub Actionsでの定期チェック等)は将来のフォローアップとする。

## Risks / Trade-offs

- [Risk] 新規リポジトリの作成はGitHub上に恒久的に残る公開リポジトリを増やす操作であり、誤って実行すると削除・改名が手間になる → [Mitigation] 実行前に必ずユーザーに確認する。リポジトリ名・公開設定(public)を明示してから作成する
- [Risk] Cask経由でも検疫属性が自動除去されないため、「Homebrewでインストールすれば警告が出ない」という誤解を招く可能性がある → [Mitigation] README・Caskのcaveats両方で、Gatekeeper警告は変わらず発生しうることを明記する
- [Trade-off] Cask更新が手動である限り、リリースを忘れてCaskが古いバージョンを指し続けるリスクがある。リリース頻度が低い現状では許容し、将来的な自動化はフォローアップ課題とする

## 1. tapリポジトリの作成(要ユーザー確認)

- [x] 1.1 **【ユーザー確認】** `kuchida1981/homebrew-bitwarden-quickaccess` という名前で新規の公開GitHubリポジトリを作成してよいか、実行前に確認する。ユーザー承認済み
- [x] 1.2 確認が取れたら `gh repo create kuchida1981/homebrew-bitwarden-quickaccess --public --description "Homebrew tap for bw-quickaccess"` でリポジトリを作成する。作成済み: https://github.com/kuchida1981/homebrew-bitwarden-quickaccess
- [x] 1.3 ローカルにclone(または `gh repo clone`)し、`Casks/` ディレクトリを作成する。ユーザーが `/Users/kosuke/projects/homebrew-bitwarden-quickaccess` にクローン、`Casks/` はClaude Codeが作成

## 2. Cask定義の作成

- [x] 2.1 最新リリースのタグとアセットのsha256を取得する: `gh release view --repo kuchida1981/bitwarden-quickaccess --json tagName,assets`。取得結果: `tagName: v1.0.0`, `sha256: 6dd69706f9c1032b98482d296fc6ad169d5bebaf39e762ba40dbd64b8bb2c77e`
- [x] 2.2 `Casks/bw-quickaccess.rb` を作成する(design.md 決定2〜5参照)。`version`・`sha256` は2.1で取得した値を使う。`url` は `https://github.com/kuchida1981/bitwarden-quickaccess/releases/download/#{tagName}/bw-quickaccess_aarch64.app.tar.gz` の形。`depends_on arch: :arm64` を指定する。`caveats` ブロックにGatekeeper対処法(右クリックで開く / `--no-quarantine`)を記載する
- [x] 2.3 `brew audit --cask bw-quickaccess`(tapリポジトリのローカルパスを一時的にtapして実行するか、`brew audit --cask ./Casks/bw-quickaccess.rb` 相当の方法)でlintし、指摘があれば修正する。`brew style`で「macOS専用Caskには`depends_on :macos`が必要」という指摘があり `depends_on macos: :big_sur` を追加して解消。`brew audit`は無指摘で通過。また新規発見: Homebrew 6以降は非公式tapに対する明示的な信頼(`brew trust`)が必要になっており、README(タスク2.5)にその案内も追記した
- [x] 2.4 `brew tap kuchida1981/bitwarden-quickaccess` → `brew install --cask bw-quickaccess` で実際にインストールできることを確認する(実機確認が必要)。Claude Codeが実行し、`/Applications/bw-quickaccess.app` へのインストールと、想定通り検疫属性(`com.apple.quarantine`)が付与されGatekeeper警告が発生することを確認済み
- [x] 2.5 tapリポジトリにREADME.mdを追加し、design.md 決定6のCask更新手順(リリースごとにtagName/sha256を取得して更新する手順)を記載する。`brew trust`の案内も追記
- [x] 2.6 tapリポジトリの変更をコミット・プッシュする(ユーザー承認の上でClaude Codeがコミット・プッシュ)

## 3. 本リポジトリのREADME更新

- [x] 3.1 README.md の `## Install` セクションに、既存の「Option 1: GitHub Releases」「Option 2: セルフビルド」と並べて「Option: Homebrew」を追加し、`brew tap kuchida1981/bitwarden-quickaccess` と `brew install --cask bw-quickaccess` のコマンド、Gatekeeper警告に関する注記を記載する。Homebrewを新Option 1とし、既存の2項目をOption 2/3に繰り下げ
- [x] 3.2 README.ja.md にも同内容を日本語で追加する
- [x] 3.3 本リポジトリへの変更(README.md, README.ja.md)をコミットする

## 4. 動作確認・仕上げ

- [x] 4.1 実機で `brew tap kuchida1981/bitwarden-quickaccess && brew install --cask bw-quickaccess` を実行し、`/Applications` にアプリがインストールされ起動できることを確認する(2.4でインストール、Gatekeeperバイパスを含む起動確認はユーザーが実施。起動成功を確認済み)
- [x] 4.2 `specs/homebrew-distribution/spec.md` の各シナリオ(README記載内容)が満たされていることを確認する

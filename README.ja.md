# bw-quickaccess

[Read this in English](README.md) / 日本語版

Bitwarden向けの、1Password Quick Access 相当のメニューバー常駐アプリです。`bw`(Bitwarden CLI)の `bw serve` と [Tauri](https://tauri.app/) を組み合わせて実装されています。どこからでもグローバルホットキーを押すだけでvaultを検索し、ユーザー名・パスワード・TOTPをクリップボードにコピーしたり、アイテムのURLをブラウザで開いたりできます。コピーした内容は、他の値を上書きコピーしていない限り、30秒後(またはvaultをロックした時点)に自動的にクリップボードからクリアされます。

> **旧ターミナル版(TUI)をお使いだった方へ**: 下記の[旧TUIからの移行](#旧tuiからの移行)を参照してください。

## 必要なもの

- macOS(Linux対応は将来のリリースで予定していますが、現時点では未対応です)
- [`bw`(Bitwarden CLI)](https://bitwarden.com/help/cli/) — `bw login` 済みであること(vaultはロック状態で構いません。アプリ側でアンロックできます)

### Bitwarden CLI (`bw`) が見つからない場合

Homebrew 経由で Bitwarden CLI をインストールしている場合(`/opt/homebrew/bin/bw` または `/usr/local/bin/bw`)、自動的に検出されるため通常は追加の設定は不要です。

一方、Node バージョン管理ツール(nvm, volta, asdf 等)経由で `npm install -g @bitwarden/cli` を実行した場合や、ネイティブバイナリを手動配置した場合は、既知のパスと異なるため設定ファイルで `bw` の絶対パスを指定する必要があります。

設定ファイル `$XDG_CONFIG_HOME/bw-quickaccess/bw_path.txt`(`XDG_CONFIG_HOME` が未設定の場合は `~/.config/bw-quickaccess/bw_path.txt`)を作成し、`bw` 実行ファイルの絶対パスを1行で記述してください。

普段お使いのシェル上で `which bw` を実行すると `bw` の絶対パスが出力されるため、以下のように設定ファイルを作成できます:

```bash
mkdir -p ~/.config/bw-quickaccess
echo "$(which bw)" > ~/.config/bw-quickaccess/bw_path.txt
```

**セルフビルド**する場合は、追加で以下が必要です:
- [Rust toolchain](https://www.rust-lang.org/tools/install)(stable、`rustup` 経由)
- [Tauri CLI](https://v2.tauri.app/reference/cli/): `cargo install tauri-cli --locked`

## インストール

### 方法1: Homebrew

```bash
brew tap kuchida1981/bitwarden-quickaccess
brew install --cask bw-quickaccess
```

このビルドは **Apple Silicon(arm64)専用**です。Intel Macは対象外です。Homebrew 6以降では、第三者のtapを初めて使う際に信頼(trust)の確認を求められることがあります:

```bash
brew trust --tap kuchida1981/bitwarden-quickaccess
```

このアプリは**コード署名・notarizationされていません**。Homebrew経由でインストールしてもGatekeeperを回避できるわけではないため、初回起動時には方法2の手順3と同様にFinderで右クリック→「開く」を選ぶか、`brew install --cask --no-quarantine bw-quickaccess` で入れ直してください。

### 方法2: GitHub Releasesからダウンロード

1. [Releasesページ](https://github.com/kuchida1981/bitwarden-quickaccess/releases)から `Bitwarden Quick Access_aarch64.app.tar.gz` をダウンロードする。このビルドは **Apple Silicon(arm64)専用**です。Intel Macは現時点でビルド済みリリースの対象外です(方法3のセルフビルドを利用してください)
2. 展開して(Finderでダブルクリック、または `tar -xzf "Bitwarden Quick Access_aarch64.app.tar.gz"`)、`Bitwarden Quick Access.app` を `/Applications`(お好きな場所でも構いません)に移動する
3. このアプリは**コード署名・notarizationされていません**。初回起動時、macOSのGatekeeperが「開発元を確認できません」という警告を出して起動をブロックします。以下の手順で起動してください:
   - Finderで `Bitwarden Quick Access.app` を右クリック(またはControl+クリック)し、**「開く」**を選択、表示されるダイアログでも**「開く」**を選ぶ
   - この操作は初回のみ必要です。以降は通常どおり起動できます

### 方法3: ソースからセルフビルド

```bash
git clone https://github.com/kuchida1981/bitwarden-quickaccess.git
cd bitwarden-quickaccess/app/src-tauri
cargo tauri build
```

生成された `.app` は `target/release/bundle/macos/Bitwarden Quick Access.app` に配置されます。お好みで `/Applications` に移動してください。

開発時(配布用バンドルを作らず、その場でアプリを実行する場合):

```bash
cd app/src-tauri
cargo run
```

## 使い方

1. `Bitwarden Quick Access.app` を起動します。起動時にDockアイコンやウィンドウは表示されません。メニューバーのアイコンを探してください
2. どこからでも **⇧⌘Space**(Shift+Cmd+Space)を押すとポップアップが開閉します
3. vaultがロックされている場合は、マスターパスワードを入力してアンロックします
4. 入力するとインクリメンタルに検索されます。**↑ / ↓** キーでフォーカスする行を移動できます
5. 行にフォーカスした状態で、以下のショートカットが使えます:

   | ショートカット | 動作 |
   |---|---|
   | `⌘C` | ユーザー名をコピー |
   | `⌘⇧C` | パスワードをコピー |
   | `⌥⌘C` | TOTPコードをコピー |
   | `Enter` | アイテムのURLをデフォルトブラウザで開く |

   アクション実行後、ポップアップは自動的に閉じます
6. ポップアップは、フォーカスを失った場合(別の場所をクリックした場合等)も自動的に閉じます

### メニューバーアイコン

トレイアイコンをクリックすると、現在のロック状態・グローバルホットキーの登録状況の確認、**ログイン時自動起動**のオン/オフ切り替え、インストール済みバージョンの確認、アプリの終了ができます。

### 自動ロック

vaultは、15分間操作(検索・コピー・ブラウザ起動)がないと自動的に再ロックされます。旧TUIのセッションTTLの挙動を踏襲したものです。現時点ではこのタイムアウト値を変更するUIはありません。

### 表示言語

UIの文言(メニューバー・ポップアップ)は、macOSのシステム言語に従います。システム言語が日本語なら日本語、それ以外なら英語で表示されます。アプリ内での言語切り替えUIはなく、システム言語を変更した場合はアプリの再起動が必要です。

## 旧TUIからの移行

このGUIへの刷新に伴い、従来のターミナル向けツール(`bin/bw-quickaccess`、`install.sh` でインストールするもの)はこのリポジトリから削除されました。以前 `curl` ワンライナーでインストールしていた場合、**自動的には削除されません**。以下の手順で手動削除してください:

```bash
rm "$HOME/.local/bin/bw-quickaccess"
```

(`--prefix` オプションでインストール先を変更していた場合は、`$PREFIX/bin/bw-quickaccess` のように読み替えてください。)

その後、上記いずれかの方法で新しいGUIアプリをインストールしてください。

## スコープ外

- Linux対応(将来のリリースで予定)
- コード署名・notarization
- アプリ内での言語切り替えUI(UIの表示言語はmacOSのシステム言語に従います。[表示言語](#表示言語)を参照)
- アイドルロックのタイムアウト値やホットキーの変更設定

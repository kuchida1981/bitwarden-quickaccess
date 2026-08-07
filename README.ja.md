# bw-quickaccess

[Read this in English](README.md) / 日本語版

`bw`(Bitwarden CLI)・`jq`・`fzf` を組み合わせた、1Password Quick Access 相当のターミナル向けクイックアクセスツールです。vault アイテムをインクリメンタルサーチし、ユーザー名・パスワード・TOTP をクリップボードへコピーできます。

## 必要なもの

- macOS、または デスクトップ GUI 環境(GNOME Keyring / KWallet 等が動作している)の Linux
- [`bw`(Bitwarden CLI)](https://bitwarden.com/help/cli/) — `bw login` 済みであること
- `jq`
- `fzf`(0.73.0 以上)
- クリップボードコピーコマンド
  - macOS: `pbcopy`(標準搭載)
  - Linux(Wayland): `wl-copy`
  - Linux(X11): `xclip` または `xsel`
- OS キーチェーン連携コマンド(session token のキャッシュに使用)
  - macOS: `security`(標準搭載)
  - Linux: `secret-tool`(`libsecret-tools` パッケージ)

また、インストールの方法に応じて以下のツールが必要です。
- **インストーラー(install.sh)を使用する場合**
  - `curl` (git は不要です)
- **ソースから clone して使用する場合**
  - `git`

不足しているツールがある場合、起動時にインストール方法を案内した上でエラー終了します。

検索画面・フィールド選択画面はフルスクリーン表示(ターミナルの alternate screen buffer を使用)になります。画面表示中はターミナルのスクロールバックが一時的に隠れ、終了時に元の画面内容へ復元されます。

## インストール

以下のコマンドを実行することで、簡単にインストールできます。

```sh
curl -fsSL https://raw.githubusercontent.com/kuchida1981/bitwarden-quickaccess/main/install.sh | bash
```

デフォルトでは、ユーザー権限で `~/.local/bin/bw-quickaccess` にインストールされます。

### オプション指定

インストール時のオプションを指定する場合は、以下のように実行します。

- **インストール先の変更 (`--prefix`)**
  デフォルトのインストール先を変更したい場合は `--prefix` オプションを指定します。
  ```sh
  curl -fsSL https://raw.githubusercontent.com/kuchida1981/bitwarden-quickaccess/main/install.sh | bash -s -- --prefix /opt/bwqa
  ```
  この例では `/opt/bwqa/bin/bw-quickaccess` にインストールされます。

- **特定バージョンのインストール (`--version`)**
  最新版以外の特定バージョンをインストールしたい場合は `--version` オプションを指定します。
  ```sh
  curl -fsSL https://raw.githubusercontent.com/kuchida1981/bitwarden-quickaccess/main/install.sh | bash -s -- --version v0.1.0
  ```

### アップデート

アップデートを行うには、インストール時と同じ curl コマンドを再実行します。再実行すると、旧バージョンから新バージョンへの更新メッセージが表示されます。

現在インストールされているバージョンは、以下のコマンドで確認できます。

```sh
bw-quickaccess --version
```
または、インストール先が PATH に通っていない場合は直接実行して確認します。
```sh
~/.local/bin/bw-quickaccess --version
```

### アンインストール

インストールした `bw-quickaccess` を削除するには、実行ファイルを削除します。

```sh
rm ~/.local/bin/bw-quickaccess
```

`--prefix` オプションでインストール先を変更した場合は、以下のように削除します。

```sh
rm <prefix>/bin/bw-quickaccess
```

## 使い方

インストールして使用する場合:
```sh
bw-quickaccess
```
※ `~/.local/bin` などのインストール先に PATH が通っている必要があります。通っていない場合は `~/.local/bin/bw-quickaccess` のようにフルパスで実行してください。

ソースから clone して直接実行する場合(開発者向けなど):
```sh
bin/bw-quickaccess
```

1. 検索画面(fzf)で vault アイテムをインクリメンタルサーチします
   - `Enter`: アイテムを選択してフィールド選択画面へ進む
   - `ctrl-r`: 絞り込んだアイテムのパスワードを直接コピー(画面はそのまま)
   - `ctrl-o`: 絞り込んだアイテムのユーザー名を直接コピー(画面はそのまま)
   - `ctrl-t`: 絞り込んだアイテムの TOTP を直接コピー(画面はそのまま)
2. フィールド選択画面で、コピーしたいフィールドを選びます
   - `Enter`: 選択中の行をコピー
   - `ctrl-r`: パスワードを直接コピー
   - `ctrl-o`: ユーザー名を直接コピー
   - `ctrl-t`: TOTP を直接コピー
   - コピーしても画面は閉じないため、同じアイテムの別フィールドを続けてコピーできます
   - `Esc`: 検索画面へ戻る
   - `q`: ツールを終了する
3. 次回起動時は、直前に選択したアイテムのフィールド選択画面から始まります(検索をスキップ)。別のアイテムを探したい場合は `Esc` で検索画面に戻ってください

### session(ログイン状態)について

初回実行時は `bw unlock` のマスターパスワード入力を求められます。取得した session token は OS のキーチェーンにキャッシュされ、既定 15 分(`BWQA_SESSION_TTL_SECONDS` 環境変数で変更可能)以内であれば再入力を求められません。

キャッシュされた session を破棄したい場合:

インストールして使用する場合:
```sh
bw-quickaccess lock
```

ソースから直接実行する場合:
```sh
bin/bw-quickaccess lock
```

### 表示言語について

CLI のメッセージは `LANG`/`LC_ALL` 環境変数から日本語・英語を自動判定します(`ja` で始まらない場合は英語)。`BWQA_LANG` 環境変数(`ja` または `en`)を設定すると明示的に切り替えられます。

```sh
BWQA_LANG=en bw-quickaccess
```

### スコープ外の機能

- Bitwarden デスクトップアプリへの deep link 連携(アプリ側が特定アイテムへの直接ナビゲーションに未対応のため)
- クリップボードの自動クリア
- Linux のヘッドレス/SSH 専用環境のサポート

## 開発者向け: テストの実行

`lib/*.sh` の単体テストは [bats-core](https://github.com/bats-core/bats-core) で書かれています。静的解析には [shellcheck](https://www.shellcheck.net/) を使用しています(除外ルールはリポジトリ直下の `.shellcheckrc` を参照)。

### セットアップ

```sh
# macOS
brew install bats-core shellcheck

# Linux(Debian/Ubuntu 系)
sudo apt-get install -y bats shellcheck
```

### 実行

```sh
# 構文チェック
bash -n bin/bw-quickaccess
for f in lib/*.sh; do bash -n "$f"; done

# 静的解析(プロダクションコードは -x でクロスファイル解析、テストコードは単体で解析)
shellcheck -x bin/bw-quickaccess
shellcheck test/helpers/*.bash test/lib/*.bats

# 単体テスト
bats test/lib/*.bats
```

GitHub Actions(`.github/workflows/ci.yml`)で `macos-latest` / `ubuntu-latest` の両方に対して push・pull request のたびに同じチェックを自動実行しています。

詳細な要件・設計は `openspec/changes/add-quickaccess-cli/` を参照してください。

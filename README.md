# bw-quickaccess

`bw`(Bitwarden CLI)・`jq`・`fzf` を組み合わせた、1Password Quick Access 相当のターミナル向けクイックアクセスツールです。vault アイテムをインクリメンタルサーチし、ユーザー名・パスワード・TOTP をクリップボードへコピーできます。

## 必要なもの

- macOS、または デスクトップ GUI 環境(GNOME Keyring / KWallet 等が動作している)の Linux
- [`bw`(Bitwarden CLI)](https://bitwarden.com/help/cli/) — `bw login` 済みであること
- `jq`
- `fzf`(0.37.0 以上)
- クリップボードコピーコマンド
  - macOS: `pbcopy`(標準搭載)
  - Linux(Wayland): `wl-copy`
  - Linux(X11): `xclip` または `xsel`
- OS キーチェーン連携コマンド(session token のキャッシュに使用)
  - macOS: `security`(標準搭載)
  - Linux: `secret-tool`(`libsecret-tools` パッケージ)

不足しているツールがある場合、起動時にインストール方法を案内した上でエラー終了します。

## 使い方

```sh
bin/bw-quickaccess
```

1. 検索画面(fzf)で vault アイテムをインクリメンタルサーチします
   - `Enter`: アイテムを選択してフィールド選択画面へ進む
   - `alt-p`: 絞り込んだアイテムのパスワードを直接コピー(画面はそのまま)
   - `alt-u`: 絞り込んだアイテムのユーザー名を直接コピー(画面はそのまま)
   - `ctrl-t`: 絞り込んだアイテムの TOTP を直接コピー(画面はそのまま)
2. フィールド選択画面で、コピーしたいフィールドを選びます
   - `Enter`: 選択中の行をコピー
   - `alt-p`: パスワードを直接コピー
   - `alt-u`: ユーザー名を直接コピー
   - `ctrl-t`: TOTP を直接コピー
   - コピーしても画面は閉じないため、同じアイテムの別フィールドを続けてコピーできます
   - `Esc`: 検索画面へ戻る
   - `q`: ツールを終了する
3. 次回起動時は、直前に選択したアイテムのフィールド選択画面から始まります(検索をスキップ)。別のアイテムを探したい場合は `Esc` で検索画面に戻ってください

`alt-p`/`alt-u` はターミナルエミュレータの Meta キー送信設定に依存します。macOS 標準の Terminal.app では、Profile > Keyboard の「Use Option as Meta Key」を有効にしないと動作しない場合があります。動作しない場合は該当設定を有効にするか、iTerm2 等の別のターミナルエミュレータをご利用ください。

### session(ログイン状態)について

初回実行時は `bw unlock` のマスターパスワード入力を求められます。取得した session token は OS のキーチェーンにキャッシュされ、既定 15 分(`BWQA_SESSION_TTL_SECONDS` 環境変数で変更可能)以内であれば再入力を求められません。

キャッシュされた session を破棄したい場合:

```sh
bin/bw-quickaccess lock
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

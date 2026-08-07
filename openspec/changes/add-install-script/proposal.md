## Why

現在 bw-quickaccess を導入するには本リポジトリを clone し、`bin/bw-quickaccess` を直接実行する(または手動で PATH に通す)以外の方法がない。`bin/bw-quickaccess` は自身のパスから算出した `BWQA_ROOT` 経由で `lib/*.sh` を相対 source しているため、`bin/` と `lib/` が同じ階層に存在しないと動作せず、単純なファイルコピーやインストーラーでの配布ができない(issue #7)。もう少し手軽に導入できる手段(`curl ... | bash` 形式の install.sh)を用意する。

## What Changes

- ビルド時に `bin/bw-quickaccess` + `lib/*.sh` を単一の自己完結スクリプトへ連結する `script/build.sh` を追加する
- GitHub Release が公開(`release: published`)された際に、ビルド済みバンドルをそのリリースにアセットとして添付する CI ワークフロー(`.github/workflows/release.yml`)を追加する。release/tag の作成自体は人間が `gh release create` で行う(CI は release を作成しない)
- リポジトリ直下に `install.sh` を追加する。`curl -fsSL <raw URL> | bash` で実行でき、デフォルトでユーザー権限(`~/.local/bin`)にインストールする。`--prefix` オプションでインストール先ルートを変更できる
- アップデート手段として専用スクリプトは設けず、`install.sh` を再実行することで既存インストールを上書き更新できるようにする。更新前後のバージョンをユーザーに提示できるよう、`bin/bw-quickaccess` に `--version`/`-v` オプションを追加し、ビルド時にリリースタグをバージョン文字列として埋め込む
- README にインストール手順(curl one-liner、`--prefix` オプション、アンインストール手順、アップデート手順)を追記する

## Capabilities

### New Capabilities
- `installation`: `install.sh` によるエンドユーザー向けインストール体験(curlワンライナー実行、`--prefix` オプション、バージョン指定、PATH 未設定時の警告表示)を規定する
- `release-packaging`: `bin/`+`lib/*.sh` を単一実行ファイルへバンドルするビルド処理と、GitHub Release 公開時にそのバンドルをアセットとして添付する CI の振る舞いを規定する

### Modified Capabilities
(なし。既存のランタイム挙動(vault検索・フィールドコピー・session管理等)には変更がない。`--version` オプションの追加は新規の振る舞いであり、既存要件の変更ではないため `release-packaging` capability への ADDED Requirement として扱う)

## Impact

- 追加: `install.sh`(リポジトリ直下)、`script/build.sh`、`.github/workflows/release.yml`
- 変更: `README.md`(インストール手順・アップデート手順の追記)、`bin/bw-quickaccess`(`--version`/`-v` オプションの追加。既存のフィールド検索・コピー等のロジックには影響しない)
- 前提作業: 初回リリースタグ(`v0.1.0` 想定、milestone 0.1 に対応)をこの change の実装完了後に人手で作成する必要がある

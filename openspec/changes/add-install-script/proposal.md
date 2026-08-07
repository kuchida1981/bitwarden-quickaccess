## Why

現在 bw-quickaccess を導入するには本リポジトリを clone し、`bin/bw-quickaccess` を直接実行する(または手動で PATH に通す)以外の方法がない。`bin/bw-quickaccess` は自身のパスから算出した `BWQA_ROOT` 経由で `lib/*.sh` を相対 source しているため、`bin/` と `lib/` が同じ階層に存在しないと動作せず、単純なファイルコピーやインストーラーでの配布ができない(issue #7)。もう少し手軽に導入できる手段(`curl ... | bash` 形式の install.sh)を用意する。

## What Changes

- ビルド時に `bin/bw-quickaccess` + `lib/*.sh` を単一の自己完結スクリプトへ連結する `script/build.sh` を追加する
- GitHub Release が公開(`release: published`)された際に、ビルド済みバンドルをそのリリースにアセットとして添付する CI ワークフロー(`.github/workflows/release.yml`)を追加する。release/tag の作成自体は人間が `gh release create` で行う(CI は release を作成しない)
- リポジトリ直下に `install.sh` を追加する。`curl -fsSL <raw URL> | bash` で実行でき、デフォルトでユーザー権限(`~/.local/bin`)にインストールする。`--prefix` オプションでインストール先ルートを変更できる
- README にインストール手順(curl one-liner、`--prefix` オプション、アンインストール手順)を追記する

## Capabilities

### New Capabilities
- `installation`: `install.sh` によるエンドユーザー向けインストール体験(curlワンライナー実行、`--prefix` オプション、バージョン指定、PATH 未設定時の警告表示)を規定する
- `release-packaging`: `bin/`+`lib/*.sh` を単一実行ファイルへバンドルするビルド処理と、GitHub Release 公開時にそのバンドルをアセットとして添付する CI の振る舞いを規定する

### Modified Capabilities
(なし。既存の `bw-session-management` 等のランタイム挙動には変更がない。バンドル後も `source` 連鎖と同一の実行結果になることが要件)

## Impact

- 追加: `install.sh`(リポジトリ直下)、`script/build.sh`、`.github/workflows/release.yml`
- 変更: `README.md`(インストール手順の追記)
- 影響なし: `bin/bw-quickaccess` / `lib/*.sh` 自体のロジック(バンドルはビルド時の連結のみで、ソースコードの変更は不要)
- 前提作業: 初回リリースタグ(`v0.1.0` 想定、milestone 0.1 に対応)をこの change の実装完了後に人手で作成する必要がある

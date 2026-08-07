## 1. ビルドスクリプト(バンドル生成)

- [ ] 1.1 `script/build.sh` を新規作成し、`bin/bw-quickaccess` の `source` 行を除いた本体と `lib/common.sh` → `preflight.sh` → `clipboard.sh` → `session.sh` → `search.sh` → `fields.sh` の順に連結して、単一の実行可能スクリプト(デフォルト出力先: `dist/bw-quickaccess` 等)を生成するようにする。出力ファイルには実行権限を付与する
- [ ] 1.2 生成物に対して `bash -n` を実行し、構文エラーが無いことを確認する
- [ ] 1.3 生成物に対して `shellcheck -x` を実行し、既存の `.shellcheckrc` の除外ルールに沿って警告が出ないことを確認する
- [ ] 1.4 手元で `bin/` と `lib/` を含まない一時ディレクトリに生成物のみを配置し、`bw-session-management` / `vault-item-search` / `credential-clipboard-copy` 等の主要な既存テストシナリオ(vault 検索・フィールドコピー・`lock` サブコマンド・fzf からの `__copy-field` 再帰呼び出し)を手動確認し、clone 実行時と同一の挙動になることを検証する

## 2. リリース CI ワークフロー

- [ ] 2.1 `.github/workflows/release.yml` を新規作成し、`on: release: types: [published]` をトリガーに設定する
- [ ] 2.2 ワークフロー内で `github.event.release.tag_name` が指す commit を checkout し、`script/build.sh` を実行してバンドルをビルドするステップを追加する
- [ ] 2.3 ビルド成果物に対して `bash -n` / `shellcheck` を実行するステップを追加する(タスク 1.2/1.3 と同内容を CI 上でも実施)
- [ ] 2.4 `gh release upload "${{ github.event.release.tag_name }}" <ビルド成果物> --clobber` で、公開済みの release にバンドルをアセットとして添付するステップを追加する(release 自体は作成しない)
- [ ] 2.5 ワークフローに `permissions: contents: write`(release へのアセット添付に必要な権限)を設定する

## 3. install.sh

- [ ] 3.1 リポジトリ直下に `install.sh` を新規作成する。デフォルトで `PREFIX=~/.local` を設定し、`--prefix <path>` オプションでオーバーライドできるようにする
- [ ] 3.2 バージョン解決ロジックを実装する: オプション未指定時は `https://github.com/<owner>/<repo>/releases/latest/download/bw-quickaccess` を、`--version <tag>` 指定時は `https://github.com/<owner>/<repo>/releases/download/<tag>/bw-quickaccess` を curl でダウンロードする
- [ ] 3.3 ダウンロードしたファイルを `$PREFIX/bin/bw-quickaccess` に配置し `chmod +x` する。`$PREFIX/bin` が存在しない場合は作成する
- [ ] 3.4 インストール完了後、`$PREFIX/bin` が現在の `PATH` に含まれているか確認し、含まれていなければ `export PATH="$PREFIX/bin:$PATH"` を追加するよう促す警告メッセージを表示する(シェル rc ファイルの自動編集はしない)
- [ ] 3.5 `-h`/`--help` オプションで使い方(オプション一覧)を表示できるようにする
- [ ] 3.6 `bash -n install.sh` と `shellcheck install.sh` を実行し、構文・静的解析エラーが無いことを確認する
- [ ] 3.7 `git` コマンドが存在しない環境でも install.sh が完結すること(curl のみに依存していること)を確認する

## 4. ドキュメント更新

- [ ] 4.1 README.md に curl ワンライナーによるインストール手順(`curl -fsSL <URL> | bash`)を追記する
- [ ] 4.2 README.md に `--prefix` オプション・バージョン指定オプションの説明を追記する
- [ ] 4.3 README.md にアンインストール手順(`rm $PREFIX/bin/bw-quickaccess` 等)を追記する
- [ ] 4.4 README.md の「必要なもの」節に、install.sh 実行には `curl` のみが必要で `git` は不要である旨を追記する(既存の clone 前提の記述と整合させる)

## 5. 初回リリースと動作検証

- [ ] 5.1 CI ワークフローの動作を、本番タグを切る前に検証する(例: 一時的なプレリリース版タグでの動作確認)
- [ ] 5.2 検証用に作成した一時タグ・release がある場合は削除し、リポジトリをクリーンな状態に戻す
- [ ] 5.3 `v0.1.0`(milestone 0.1 に対応)のタグ・release を人間が作成し(`gh release create v0.1.0 --generate-notes`)、CI がバンドルを正しく添付することを確認する
- [ ] 5.4 実際に `curl -fsSL <install.sh の URL> | bash` を実行し、`bw-quickaccess` コマンドがエンドツーエンドで動作することを確認する

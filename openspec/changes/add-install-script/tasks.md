## 1. bin/bw-quickaccess へのバージョン表示オプション追加

- [x] 1.1 `bin/bw-quickaccess` に `: "${BWQA_VERSION:=dev}"`(未設定時のみ `dev` を設定)を追加する
- [x] 1.2 引数ディスパッチ(`case` 文)に `-v`/`--version` を追加し、`bw-quickaccess $BWQA_VERSION` の形式で標準出力に出力して正常終了するようにする
- [x] 1.3 `bwqa_print_usage` のヘルプ文言に `-v`/`--version` の説明を追加する
- [x] 1.4 `bash -n bin/bw-quickaccess` と `shellcheck -x bin/bw-quickaccess` を実行し、既存チェックが通ることを確認する
- [x] 1.5 既存の bats テストに影響が無いことを確認し、必要であれば `--version` 出力の簡単なテストケースを追加する(既存の bats テスト構成は lib/*.sh を対象にしており、bin/bw-quickaccess 自体を対象にしたテストは存在しないため新規テストは追加せず、既存54テストが全てパスすることのみ確認した)

## 2. ビルドスクリプト(バンドル生成)

- [x] 2.1 `script/build.sh` を新規作成し、`bin/bw-quickaccess` の `source` 行を除いた本体と `lib/common.sh` → `preflight.sh` → `clipboard.sh` → `session.sh` → `search.sh` → `fields.sh` の順に連結して、単一の実行可能スクリプト(デフォルト出力先: `dist/bw-quickaccess` 等)を生成するようにする。出力ファイルには実行権限を付与する
- [x] 2.2 環境変数 `VERSION` が指定されている場合、生成物の先頭付近(`: "${BWQA_VERSION:=dev}"` より前)に `BWQA_VERSION="$VERSION"` を書き込むようにする。未指定の場合は何も書き込まず、`bin/bw-quickaccess` 側のデフォルト(`dev`)がそのまま使われるようにする
- [x] 2.3 生成物に対して `bash -n` を実行し、構文エラーが無いことを確認する
- [x] 2.4 生成物に対して `shellcheck -x` を実行し、既存の `.shellcheckrc` の除外ルールに沿って警告が出ないことを確認する
- [x] 2.5 手元で `bin/` と `lib/` を含まない一時ディレクトリに生成物のみを配置し、`bw-session-management` / `vault-item-search` / `credential-clipboard-copy` 等の主要な既存テストシナリオ(vault 検索・フィールドコピー・`lock` サブコマンド・fzf からの `__copy-field` 再帰呼び出し・`--version` 出力)を手動確認し、clone 実行時と同一の挙動になることを検証する

## 3. リリース CI ワークフロー

- [ ] 3.1 `.github/workflows/release.yml` を新規作成し、`on: release: types: [published]` をトリガーに設定する
- [ ] 3.2 ワークフロー内で `github.event.release.tag_name` が指す commit を checkout し、`VERSION="${{ github.event.release.tag_name }}"` を指定して `script/build.sh` を実行しバンドルをビルドするステップを追加する
- [ ] 3.3 ビルド成果物に対して `bash -n` / `shellcheck` を実行するステップを追加する(タスク 2.3/2.4 と同内容を CI 上でも実施)
- [ ] 3.4 `gh release upload "${{ github.event.release.tag_name }}" <ビルド成果物> --clobber` で、公開済みの release にバンドルをアセットとして添付するステップを追加する(release 自体は作成しない)
- [ ] 3.5 ワークフローに `permissions: contents: write`(release へのアセット添付に必要な権限)を設定する

## 4. install.sh

- [ ] 4.1 リポジトリ直下に `install.sh` を新規作成する。デフォルトで `PREFIX=~/.local` を設定し、`--prefix <path>` オプションでオーバーライドできるようにする
- [ ] 4.2 バージョン解決ロジックを実装する: オプション未指定時は `https://github.com/<owner>/<repo>/releases/latest/download/bw-quickaccess` を、`--version <tag>` 指定時は `https://github.com/<owner>/<repo>/releases/download/<tag>/bw-quickaccess` を curl でダウンロードする
- [ ] 4.3 上書き前に `$PREFIX/bin/bw-quickaccess` が既に存在するか確認し、存在する場合は `"$PREFIX/bin/bw-quickaccess" --version` で現在のバージョンを取得しておく
- [ ] 4.4 ダウンロードしたファイルを `$PREFIX/bin/bw-quickaccess` に配置し `chmod +x` する。`$PREFIX/bin` が存在しない場合は作成する
- [ ] 4.5 インストール後、`"$PREFIX/bin/bw-quickaccess" --version` で新バージョンを取得し、既存インストールがあった場合は `<旧バージョン> から <新バージョン> に更新しました`、無かった場合は `<新バージョン> をインストールしました` を表示する
- [ ] 4.6 インストール完了後、`$PREFIX/bin` が現在の `PATH` に含まれているか確認し、含まれていなければ `export PATH="$PREFIX/bin:$PATH"` を追加するよう促す警告メッセージを表示する(シェル rc ファイルの自動編集はしない)
- [ ] 4.7 `-h`/`--help` オプションで使い方(オプション一覧)を表示できるようにする
- [ ] 4.8 `bash -n install.sh` と `shellcheck install.sh` を実行し、構文・静的解析エラーが無いことを確認する
- [ ] 4.9 `git` コマンドが存在しない環境でも install.sh が完結すること(curl のみに依存していること)を確認する

## 5. ドキュメント更新

- [ ] 5.1 README.md に curl ワンライナーによるインストール手順(`curl -fsSL <URL> | bash`)を追記する
- [ ] 5.2 README.md に `--prefix` オプション・バージョン指定オプションの説明を追記する
- [ ] 5.3 README.md にアップデート手順(同じ curl コマンドの再実行で更新できる旨、`bw-quickaccess --version` での確認方法)を追記する
- [ ] 5.4 README.md にアンインストール手順(`rm $PREFIX/bin/bw-quickaccess` 等)を追記する
- [ ] 5.5 README.md の「必要なもの」節に、install.sh 実行には `curl` のみが必要で `git` は不要である旨を追記する(既存の clone 前提の記述と整合させる)

## 6. 初回リリースと動作検証

- [ ] 6.1 CI ワークフローの動作を、本番タグを切る前に検証する(例: 一時的なプレリリース版タグでの動作確認。`--version` の埋め込みが正しく機能することも合わせて確認する)
- [ ] 6.2 検証用に作成した一時タグ・release がある場合は削除し、リポジトリをクリーンな状態に戻す
- [ ] 6.3 `v0.1.0`(milestone 0.1 に対応)のタグ・release を人間が作成し(`gh release create v0.1.0 --generate-notes`)、CI がバンドルを正しく添付することを確認する
- [ ] 6.4 実際に `curl -fsSL <install.sh の URL> | bash` を実行し、`bw-quickaccess` コマンドがエンドツーエンドで動作することを確認する
- [ ] 6.5 同じコマンドをもう一度実行し、アップデート表示(旧バージョン→新バージョン)が正しく表示されることを確認する(将来のバージョンが出た際に実施。v0.1.0 が唯一のリリースの場合は「既にインストール済みのバージョンが表示される」動作の確認に代える)

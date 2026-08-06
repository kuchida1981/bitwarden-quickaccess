## 1. テスト基盤のセットアップ

- [ ] 1.1 `test/fixtures/bw-list-items.json`(type==1/type!=1 混在、username 有無違いを含む複数アイテム)を作成する
- [ ] 1.2 `test/fixtures/bw-get-item.json`(password/username/totp の有無パターン違いを複数用意)を作成する
- [ ] 1.3 `test/helpers/stub.bash`(一時 PATH ディレクトリの作成・ダミー実行ファイル生成・setup/teardown での PATH 復元を行う共通ヘルパー)を作成する

## 2. lib/common.sh のテスト

- [ ] 2.1 `test/lib/common.bats` を作成し、`bwqa_version_ge()` の桁上がり・等値・不足桁(例: "0.35" vs "0.35.0")のケースを検証する

## 3. lib/session.sh のテスト

- [ ] 3.1 `test/lib/session.bats` を作成し、`bwqa_session_ttl_expired()` の「ファイル無し」「TTL未満」「TTL境界」「TTL超過」「不正な内容(非数値)」のケースを検証する

## 4. lib/search.sh のテスト

- [ ] 4.1 `test/lib/search.bats` を作成し、`bwqa_bw()` を関数スタブでオーバーライドして `test/fixtures/bw-list-items.json` を返させ、`bwqa_fetch_items()` が type==1 のみを id/label 形式に整形することを検証する
- [ ] 4.2 `bwqa_fetch_items()` の label 生成ロジック(username 有無での括弧付与、改行/タブのサニタイズ)を検証するケースを追加する

## 5. lib/fields.sh のテスト

- [ ] 5.1 `test/lib/fields.bats` を作成し、`bwqa_build_field_rows()` が has_password/has_username/has_totp の組み合わせごとに正しい行を生成し、password が存在する場合は先頭行になることを検証する
- [ ] 5.2 `bwqa_bw()` を関数スタブでオーバーライドして `test/fixtures/bw-get-item.json` を返させ、`bwqa_get_item_summary()` の JSON 整形を検証する
- [ ] 5.3 PATH ダミー `bw` を用意し、`bwqa_copy_field_internal()` の正常系(値取得成功 → クリップボードコピー成功)を検証する。クリップボードコマンドはダミー実行ファイルまたは `BWQA_CLIPBOARD_CMD_ARR` の差し替えでモックする
- [ ] 5.4 `bwqa_copy_field_internal()` の異常系(値取得失敗時に `BWQA_ERROR_LOG_FILE` へ記録、不正な field 名指定時のエラー、item_id/session/field 不足時のエラー)を検証する

## 6. lib/preflight.sh のテスト

- [ ] 6.1 `test/lib/preflight.bats` を作成し、PATH ダミー実行ファイルの有無を切り替えて `bwqa_check_core_tools`(bw/jq/fzf 不足時のエラー終了)を検証する
- [ ] 6.2 PATH ダミー `fzf --version` の出力を差し替え、`bwqa_check_fzf_version()` の境界値(0.34.9 で失敗 / 0.35.0 で成功 / バージョン取得不可で失敗)を検証する
- [ ] 6.3 `uname` のダミー化と `WAYLAND_DISPLAY`/`DISPLAY` 環境変数の差し替えで、`bwqa_detect_platform()` の macOS/Linux(Wayland/X11)/非対応OS/ディスプレイ未検出のケースを検証する
- [ ] 6.4 PATH ダミー実行ファイル(`wl-copy`/`xclip`/`xsel`/`pbcopy`)の有無を切り替えて `bwqa_detect_clipboard_cmd()` の各分岐を検証する

## 7. CI ワークフロー

- [ ] 7.1 `.github/workflows/ci.yml` を作成し、`macos-latest` / `ubuntu-latest` の matrix で `bash -n bin/bw-quickaccess lib/*.sh` による構文チェックを実行するジョブを定義する
- [ ] 7.2 同ワークフローに `shellcheck` による静的解析ステップを追加する(macOS ランナーは `brew install shellcheck`、Ubuntu ランナーは標準搭載を利用)
- [ ] 7.3 同ワークフローに bats-core のインストール(macOS: `brew install bats-core`、Linux: `apt-get install -y bats` または `bats-core/bats-action`)と `bats test/lib/*.bats` の実行ステップを追加する

## 8. ドキュメント更新

- [ ] 8.1 README.md に開発者向けセクションを追加し、ローカルでのテスト実行方法(`bats-core`/`shellcheck` のインストールコマンド、`bats test/lib/*.bats` の実行方法)を記載する

## 9. 検証

- [ ] 9.1 ローカルで `bash -n bin/bw-quickaccess lib/*.sh`、`shellcheck bin/bw-quickaccess lib/*.sh`、`bats test/lib/*.bats` をすべて実行し、パスすることを確認する
- [ ] 9.2 CI ワークフローが GitHub Actions 上で `macos-latest` / `ubuntu-latest` 双方で成功することを確認する(PR 作成後 `gh pr checks --watch` で確認)

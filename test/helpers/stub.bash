# bw-quickaccess テスト共通ヘルパー
#
# 各 .bats ファイルの setup()/teardown() から `load '../helpers/stub'` で読み込んで使う。
# - PATH モック: 一時ディレクトリを PATH 先頭に追加し、ダミー実行ファイルを配置する
# - キャッシュディレクトリ: BWQA_CACHE_DIR 系の変数をテストごとの一時ディレクトリに差し替える
#
# 使い方の例(test/lib/foo.bats):
#   load '../helpers/stub'
#
#   setup() {
#     bwqa_test_stub_setup
#     source "$BWQA_LIB_DIR/common.sh"
#     source "$BWQA_LIB_DIR/foo.sh"
#   }
#
#   teardown() {
#     bwqa_test_stub_teardown
#   }
#
#   @test "..." {
#     bwqa_test_stub_cmd fzf 'printf "0.35.0\n"'
#     run some_function
#     [ "$status" -eq 0 ]
#   }

BWQA_LIB_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../lib" && pwd)"
BWQA_TEST_FIXTURES_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../fixtures" && pwd)"
export BWQA_LIB_DIR BWQA_TEST_FIXTURES_DIR

BWQA_TEST_STUB_DIR=""
BWQA_TEST_CACHE_DIR=""
BWQA_TEST_ORIG_PATH=""

# PATH モック用の一時ディレクトリと、テスト専用キャッシュディレクトリを用意する。
# PATH 上のダミー実行ファイルは別プロセスとして起動されるため、それらが参照する
# 変数は export しておく必要がある(BWQA_TEST_CACHE_DIR 等)。
bwqa_test_stub_setup() {
  BWQA_TEST_STUB_DIR="$(mktemp -d)"
  BWQA_TEST_ORIG_PATH="$PATH"
  PATH="$BWQA_TEST_STUB_DIR:$PATH"

  BWQA_TEST_CACHE_DIR="$(mktemp -d)"
  BWQA_CACHE_DIR="$BWQA_TEST_CACHE_DIR"
  BWQA_SESSION_ISSUED_AT_FILE="$BWQA_CACHE_DIR/session-issued-at"
  BWQA_LAST_ITEM_FILE="$BWQA_CACHE_DIR/last-item-id"
  BWQA_ERROR_LOG_FILE="$BWQA_CACHE_DIR/last-error.log"
  export BWQA_TEST_STUB_DIR BWQA_TEST_CACHE_DIR BWQA_CACHE_DIR \
    BWQA_SESSION_ISSUED_AT_FILE BWQA_LAST_ITEM_FILE BWQA_ERROR_LOG_FILE
}

# PATH とキャッシュディレクトリを元に戻し、一時ディレクトリを削除する。
bwqa_test_stub_teardown() {
  PATH="$BWQA_TEST_ORIG_PATH"
  [[ -n "$BWQA_TEST_STUB_DIR" && -d "$BWQA_TEST_STUB_DIR" ]] && rm -rf "$BWQA_TEST_STUB_DIR"
  [[ -n "$BWQA_TEST_CACHE_DIR" && -d "$BWQA_TEST_CACHE_DIR" ]] && rm -rf "$BWQA_TEST_CACHE_DIR"
  BWQA_TEST_STUB_DIR=""
  BWQA_TEST_CACHE_DIR=""
}

# PATH 上にダミー実行ファイルを作成する。
#   bwqa_test_stub_cmd <コマンド名> <スクリプト本体>
# 例: bwqa_test_stub_cmd fzf 'printf "0.35.0\n"'
#     bwqa_test_stub_cmd bw 'cat "$BWQA_TEST_FIXTURES_DIR/bw-list-items.json"'
bwqa_test_stub_cmd() {
  local name="$1" body="$2"
  local path="$BWQA_TEST_STUB_DIR/$name"
  {
    printf '#!/usr/bin/env bash\n'
    printf '%s\n' "$body"
  } >"$path"
  chmod +x "$path"
}

# 「コマンドが存在しない」ケースを検証するために、配置済みのダミー実行ファイルを削除する。
bwqa_test_stub_remove_cmd() {
  local name="$1"
  rm -f "$BWQA_TEST_STUB_DIR/$name"
}

# 「コマンドが存在しない」ケースを厳密に検証するため、PATH を stub ディレクトリのみに
# 制限する。通常の bwqa_test_stub_setup は既存 PATH の先頭に stub dir を追加するだけ
# なので、実行環境に本物のコマンド(bw/jq/fzf 等)がインストールされていると
# 「存在しない」状態を再現できない。制限後も lib/*.sh が内部で使う awk は、
# 呼び出し時点の実体へのパススルースタブとして stub ディレクトリに用意しておく。
#
# 注意: bwqa_test_stub_cmd 自体が chmod を必要とするため、このディレクティブは
# 各テストで必要なダミーコマンドをすべて bwqa_test_stub_cmd で作り終えた後、
# 最後に呼び出すこと(先に呼ぶと以降の bwqa_test_stub_cmd が chmod 不在で失敗する)。
bwqa_test_stub_path_only() {
  local real_awk
  real_awk="$(command -v awk)"
  bwqa_test_stub_cmd awk "exec '$real_awk' \"\$@\""
  PATH="$BWQA_TEST_STUB_DIR"
}

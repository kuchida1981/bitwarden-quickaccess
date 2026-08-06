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

BWQA_TEST_STUB_DIR=""
BWQA_TEST_CACHE_DIR=""
BWQA_TEST_ORIG_PATH=""

# PATH モック用の一時ディレクトリと、テスト専用キャッシュディレクトリを用意する。
bwqa_test_stub_setup() {
  BWQA_TEST_STUB_DIR="$(mktemp -d)"
  BWQA_TEST_ORIG_PATH="$PATH"
  PATH="$BWQA_TEST_STUB_DIR:$PATH"

  BWQA_TEST_CACHE_DIR="$(mktemp -d)"
  BWQA_CACHE_DIR="$BWQA_TEST_CACHE_DIR"
  BWQA_SESSION_ISSUED_AT_FILE="$BWQA_CACHE_DIR/session-issued-at"
  BWQA_LAST_ITEM_FILE="$BWQA_CACHE_DIR/last-item-id"
  BWQA_ERROR_LOG_FILE="$BWQA_CACHE_DIR/last-error.log"
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

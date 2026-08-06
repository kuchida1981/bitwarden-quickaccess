#!/usr/bin/env bats
# lib/preflight.sh のテスト(keychain 疎通(bwqa_check_keychain_tool)はスコープ外)

load '../helpers/stub'

setup() {
  bwqa_test_stub_setup
  source "$BWQA_LIB_DIR/common.sh"
  source "$BWQA_LIB_DIR/preflight.sh"

  # bwqa_check_core_tools 系のテストが共通で使う既定スタブ。陰性系のテストは
  # bwqa_test_stub_remove_cmd で個別に取り除いてから bwqa_test_stub_path_only
  # を呼ぶ(この2つの組み合わせで「他は揃っているが特定の1つだけ無い」を再現する)。
  bwqa_test_stub_cmd bw 'exit 0'
  bwqa_test_stub_cmd jq 'exit 0'
  bwqa_test_stub_cmd fzf '[ "$1" = "--version" ] && printf "0.35.0\n"'
}

teardown() {
  bwqa_test_stub_teardown
}

# --- 6.1 bwqa_check_core_tools ---------------------------------------------

@test "bwqa_check_core_tools: bw/jq/fzf が揃っていれば成功する" {
  bwqa_test_stub_path_only

  run bwqa_check_core_tools
  [ "$status" -eq 0 ]
}

@test "bwqa_check_core_tools: bw が無い場合はエラー終了する" {
  bwqa_test_stub_remove_cmd bw
  bwqa_test_stub_path_only

  run bwqa_check_core_tools
  [ "$status" -ne 0 ]
  [[ "$output" == *"'bw'"* ]]
}

@test "bwqa_check_core_tools: jq が無い場合はエラー終了する" {
  bwqa_test_stub_remove_cmd jq
  bwqa_test_stub_path_only

  run bwqa_check_core_tools
  [ "$status" -ne 0 ]
  [[ "$output" == *"'jq'"* ]]
}

@test "bwqa_check_core_tools: fzf が無い場合はエラー終了する" {
  bwqa_test_stub_remove_cmd fzf
  bwqa_test_stub_path_only

  run bwqa_check_core_tools
  [ "$status" -ne 0 ]
  [[ "$output" == *"'fzf'"* ]]
}

# --- 6.2 bwqa_check_fzf_version ---------------------------------------------

@test "bwqa_check_fzf_version: 要件バージョンちょうどなら成功する" {
  bwqa_test_stub_cmd fzf 'printf "0.35.0\n"'

  run bwqa_check_fzf_version
  [ "$status" -eq 0 ]
}

@test "bwqa_check_fzf_version: 要件バージョン未満なら失敗する" {
  bwqa_test_stub_cmd fzf 'printf "0.34.9\n"'

  run bwqa_check_fzf_version
  [ "$status" -ne 0 ]
  [[ "$output" == *"0.34.9"* ]]
}

@test "bwqa_check_fzf_version: バージョンを取得できない場合は失敗する" {
  bwqa_test_stub_cmd fzf 'exit 1'

  run bwqa_check_fzf_version
  [ "$status" -ne 0 ]
}

# --- 6.3 bwqa_detect_platform -------------------------------------------
# run はサブシェル内で実行されるため BWQA_OS_KIND 等の代入結果を検証できない。
# 成功系は直接呼び出し、失敗系(bwqa_die で exit する)のみ run を使う。

@test "bwqa_detect_platform: macOS を正しく判定する" {
  bwqa_test_stub_cmd uname 'printf "Darwin\n"'

  WAYLAND_DISPLAY="" DISPLAY="" bwqa_detect_platform
  [ "$BWQA_OS_KIND" = "macos" ]
}

@test "bwqa_detect_platform: Linux + Wayland を正しく判定する" {
  bwqa_test_stub_cmd uname 'printf "Linux\n"'

  WAYLAND_DISPLAY="wayland-0" DISPLAY="" bwqa_detect_platform
  [ "$BWQA_OS_KIND" = "linux" ]
  [ "$BWQA_DISPLAY_KIND" = "wayland" ]
}

@test "bwqa_detect_platform: Linux + X11 を正しく判定する" {
  bwqa_test_stub_cmd uname 'printf "Linux\n"'

  WAYLAND_DISPLAY="" DISPLAY=":0" bwqa_detect_platform
  [ "$BWQA_OS_KIND" = "linux" ]
  [ "$BWQA_DISPLAY_KIND" = "x11" ]
}

@test "bwqa_detect_platform: Linux でディスプレイが検出できない場合は失敗する" {
  bwqa_test_stub_cmd uname 'printf "Linux\n"'

  WAYLAND_DISPLAY="" DISPLAY="" run bwqa_detect_platform
  [ "$status" -ne 0 ]
}

@test "bwqa_detect_platform: 非対応 OS の場合は失敗する" {
  bwqa_test_stub_cmd uname 'printf "FreeBSD\n"'

  run bwqa_detect_platform
  [ "$status" -ne 0 ]
}

# --- 6.4 bwqa_detect_clipboard_cmd ------------------------------------------
# BWQA_OS_KIND/BWQA_DISPLAY_KIND は bwqa_detect_platform の結果を前提にする値
# なので、ここでは直接代入して bwqa_detect_clipboard_cmd 単体を検証する。

@test "bwqa_detect_clipboard_cmd: macOS では pbcopy を使う" {
  BWQA_OS_KIND="macos"
  bwqa_test_stub_cmd pbcopy 'exit 0'
  bwqa_test_stub_path_only

  bwqa_detect_clipboard_cmd
  [ "${BWQA_CLIPBOARD_CMD_ARR[0]}" = "pbcopy" ]
}

@test "bwqa_detect_clipboard_cmd: macOS で pbcopy が無い場合は失敗する" {
  BWQA_OS_KIND="macos"
  bwqa_test_stub_path_only

  run bwqa_detect_clipboard_cmd
  [ "$status" -ne 0 ]
}

@test "bwqa_detect_clipboard_cmd: Linux+Wayland では wl-copy を使う" {
  BWQA_OS_KIND="linux"
  BWQA_DISPLAY_KIND="wayland"
  bwqa_test_stub_cmd wl-copy 'exit 0'
  bwqa_test_stub_path_only

  bwqa_detect_clipboard_cmd
  [ "${BWQA_CLIPBOARD_CMD_ARR[0]}" = "wl-copy" ]
}

@test "bwqa_detect_clipboard_cmd: Linux+Wayland で wl-copy が無い場合は失敗する" {
  BWQA_OS_KIND="linux"
  BWQA_DISPLAY_KIND="wayland"
  bwqa_test_stub_path_only

  run bwqa_detect_clipboard_cmd
  [ "$status" -ne 0 ]
}

@test "bwqa_detect_clipboard_cmd: Linux+X11 では xclip を優先して使う" {
  BWQA_OS_KIND="linux"
  BWQA_DISPLAY_KIND="x11"
  bwqa_test_stub_cmd xclip 'exit 0'
  bwqa_test_stub_cmd xsel 'exit 0'
  bwqa_test_stub_path_only

  bwqa_detect_clipboard_cmd
  [ "${BWQA_CLIPBOARD_CMD_ARR[0]}" = "xclip" ]
}

@test "bwqa_detect_clipboard_cmd: Linux+X11 で xclip が無い場合は xsel にフォールバックする" {
  BWQA_OS_KIND="linux"
  BWQA_DISPLAY_KIND="x11"
  bwqa_test_stub_cmd xsel 'exit 0'
  bwqa_test_stub_path_only

  bwqa_detect_clipboard_cmd
  [ "${BWQA_CLIPBOARD_CMD_ARR[0]}" = "xsel" ]
}

@test "bwqa_detect_clipboard_cmd: Linux+X11 で xclip も xsel も無い場合は失敗する" {
  BWQA_OS_KIND="linux"
  BWQA_DISPLAY_KIND="x11"
  bwqa_test_stub_path_only

  run bwqa_detect_clipboard_cmd
  [ "$status" -ne 0 ]
}

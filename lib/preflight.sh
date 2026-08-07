# bw-quickaccess: 起動前チェック(必須コマンド・OS/表示サーバー判定・クリップボード・keychain疎通)

BWQA_OS_KIND=""
BWQA_DISPLAY_KIND=""
BWQA_KEYCHAIN_AVAILABLE="false"
declare -a BWQA_CLIPBOARD_CMD_ARR=()

bwqa_require_cmd() {
  local cmd="$1" hint="$2"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    # shellcheck disable=SC2059
    bwqa_die "$(printf "$BWQA_MSG_PREFLIGHT_CMD_NOT_FOUND" "$cmd" "$hint")"
  fi
}

bwqa_check_fzf_version() {
  local required="0.37.0"
  local raw
  raw="$(fzf --version 2>/dev/null | awk '{print $1}')"
  if [[ -z "$raw" ]]; then
    # shellcheck disable=SC2059
    bwqa_die "$(printf "$BWQA_MSG_PREFLIGHT_FZF_VERSION_UNKNOWN" "$required")"
  fi
  if ! bwqa_version_ge "$raw" "$required"; then
    # shellcheck disable=SC2059
    bwqa_die "$(printf "$BWQA_MSG_PREFLIGHT_FZF_VERSION_TOO_OLD" "$raw" "$required")"
  fi
}

bwqa_check_core_tools() {
  bwqa_require_cmd bw "$BWQA_MSG_PREFLIGHT_BW_INSTALL_HINT"
  bwqa_require_cmd jq "$BWQA_MSG_PREFLIGHT_JQ_INSTALL_HINT"
  bwqa_require_cmd fzf "$BWQA_MSG_PREFLIGHT_FZF_INSTALL_HINT"
  bwqa_check_fzf_version
}

bwqa_detect_platform() {
  case "$(uname -s)" in
    Darwin) BWQA_OS_KIND="macos" ;;
    Linux) BWQA_OS_KIND="linux" ;;
    *)
      # shellcheck disable=SC2059
      bwqa_die "$(printf "$BWQA_MSG_PREFLIGHT_OS_UNSUPPORTED" "$(uname -s)")"
      ;;
  esac

  if [[ "$BWQA_OS_KIND" == "linux" ]]; then
    if [[ -n "${WAYLAND_DISPLAY:-}" ]]; then
      BWQA_DISPLAY_KIND="wayland"
    elif [[ -n "${DISPLAY:-}" ]]; then
      BWQA_DISPLAY_KIND="x11"
    else
      bwqa_die "$BWQA_MSG_PREFLIGHT_DISPLAY_NOT_FOUND"
    fi
  fi
}

bwqa_detect_clipboard_cmd() {
  case "$BWQA_OS_KIND" in
    macos)
      bwqa_require_cmd pbcopy "$BWQA_MSG_PREFLIGHT_MACOS_BUILTIN_HINT"
      BWQA_CLIPBOARD_CMD_ARR=(pbcopy)
      ;;
    linux)
      case "$BWQA_DISPLAY_KIND" in
        wayland)
          if command -v wl-copy >/dev/null 2>&1; then
            BWQA_CLIPBOARD_CMD_ARR=(wl-copy)
          else
            bwqa_die "$BWQA_MSG_PREFLIGHT_WL_COPY_NOT_FOUND"
          fi
          ;;
        x11)
          if command -v xclip >/dev/null 2>&1; then
            BWQA_CLIPBOARD_CMD_ARR=(xclip -selection clipboard)
          elif command -v xsel >/dev/null 2>&1; then
            BWQA_CLIPBOARD_CMD_ARR=(xsel --clipboard --input)
          else
            bwqa_die "$BWQA_MSG_PREFLIGHT_XCLIP_NOT_FOUND"
          fi
          ;;
      esac
      ;;
  esac
}

bwqa_keyring_selftest() {
  local test_value="bw-quickaccess-selftest-$$"
  if ! secret-tool store --label="bw-quickaccess selftest" bwqa-selftest probe <<<"$test_value" >/dev/null 2>&1; then
    return 1
  fi
  local got
  got="$(secret-tool lookup bwqa-selftest probe 2>/dev/null || true)"
  secret-tool clear bwqa-selftest probe >/dev/null 2>&1 || true
  [[ "$got" == "$test_value" ]]
}

bwqa_check_keychain_tool() {
  case "$BWQA_OS_KIND" in
    macos)
      bwqa_require_cmd security "$BWQA_MSG_PREFLIGHT_MACOS_BUILTIN_HINT"
      BWQA_KEYCHAIN_AVAILABLE="true"
      ;;
    linux)
      if ! command -v secret-tool >/dev/null 2>&1; then
        bwqa_die "$BWQA_MSG_PREFLIGHT_SECRET_TOOL_NOT_FOUND"
      fi
      if bwqa_keyring_selftest; then
        BWQA_KEYCHAIN_AVAILABLE="true"
      else
        BWQA_KEYCHAIN_AVAILABLE="false"
        bwqa_log "$BWQA_MSG_PREFLIGHT_KEYRING_SELFTEST_FAILED"
      fi
      ;;
  esac
}

bwqa_run_preflight() {
  bwqa_check_core_tools
  bwqa_detect_platform
  bwqa_detect_clipboard_cmd
  bwqa_check_keychain_tool
}

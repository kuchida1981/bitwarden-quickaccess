# bw-quickaccess: 起動前チェック(必須コマンド・OS/表示サーバー判定・クリップボード・keychain疎通)

BWQA_OS_KIND=""
BWQA_DISPLAY_KIND=""
BWQA_KEYCHAIN_AVAILABLE="false"
declare -a BWQA_CLIPBOARD_CMD_ARR=()

bwqa_require_cmd() {
  local cmd="$1" hint="$2"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    bwqa_die "必須コマンド '${cmd}' が見つかりません。${hint}"
  fi
}

bwqa_check_fzf_version() {
  local required="0.35.0"
  local raw
  raw="$(fzf --version 2>/dev/null | awk '{print $1}')"
  if [[ -z "$raw" ]]; then
    bwqa_die "fzf のバージョンを取得できませんでした。fzf ${required} 以上をインストールしてください(例: brew install fzf)。"
  fi
  if ! bwqa_version_ge "$raw" "$required"; then
    bwqa_die "fzf のバージョンが古すぎます(検出: ${raw} / 必要: ${required} 以上)。'brew upgrade fzf' 等でアップグレードしてください。"
  fi
}

bwqa_check_core_tools() {
  bwqa_require_cmd bw "https://bitwarden.com/help/cli/ を参照してインストールしてください(例: brew install bitwarden-cli)。"
  bwqa_require_cmd jq "'brew install jq' または各ディストリのパッケージマネージャでインストールしてください。"
  bwqa_require_cmd fzf "'brew install fzf' または各ディストリのパッケージマネージャでインストールしてください。"
  bwqa_check_fzf_version
}

bwqa_detect_platform() {
  case "$(uname -s)" in
    Darwin) BWQA_OS_KIND="macos" ;;
    Linux) BWQA_OS_KIND="linux" ;;
    *) bwqa_die "サポート対象外の OS です($(uname -s))。macOS または Linux(デスクトップ環境)のみサポートします。" ;;
  esac

  if [[ "$BWQA_OS_KIND" == "linux" ]]; then
    if [[ -n "${WAYLAND_DISPLAY:-}" ]]; then
      BWQA_DISPLAY_KIND="wayland"
    elif [[ -n "${DISPLAY:-}" ]]; then
      BWQA_DISPLAY_KIND="x11"
    else
      bwqa_die "Wayland/X11 のディスプレイが検出できませんでした。デスクトップ GUI 環境で実行してください(ヘッドレス/SSH専用環境は非対応です)。"
    fi
  fi
}

bwqa_detect_clipboard_cmd() {
  case "$BWQA_OS_KIND" in
    macos)
      bwqa_require_cmd pbcopy "macOS には標準搭載されているはずです。PATH を確認してください。"
      BWQA_CLIPBOARD_CMD_ARR=(pbcopy)
      ;;
    linux)
      case "$BWQA_DISPLAY_KIND" in
        wayland)
          if command -v wl-copy >/dev/null 2>&1; then
            BWQA_CLIPBOARD_CMD_ARR=(wl-copy)
          else
            bwqa_die "wl-copy が見つかりません。'apt install wl-clipboard' 等でインストールしてください。"
          fi
          ;;
        x11)
          if command -v xclip >/dev/null 2>&1; then
            BWQA_CLIPBOARD_CMD_ARR=(xclip -selection clipboard)
          elif command -v xsel >/dev/null 2>&1; then
            BWQA_CLIPBOARD_CMD_ARR=(xsel --clipboard --input)
          else
            bwqa_die "xclip または xsel が見つかりません。'apt install xclip' 等でインストールしてください。"
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
      bwqa_require_cmd security "macOS には標準搭載されているはずです。PATH を確認してください。"
      BWQA_KEYCHAIN_AVAILABLE="true"
      ;;
    linux)
      if ! command -v secret-tool >/dev/null 2>&1; then
        bwqa_die "secret-tool が見つかりません。'apt install libsecret-tools' 等でインストールしてください。"
      fi
      if bwqa_keyring_selftest; then
        BWQA_KEYCHAIN_AVAILABLE="true"
      else
        BWQA_KEYCHAIN_AVAILABLE="false"
        bwqa_log "警告: keyring バックエンド(GNOME Keyring/KWallet 等)への疎通に失敗しました。session のキャッシュは無効化し、毎回マスターパスワードの入力を求めます。"
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

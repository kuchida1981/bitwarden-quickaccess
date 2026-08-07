# bw-quickaccess: 共有パス・ログ・キャッシュディレクトリ管理
# このファイルは source される前提で、直接実行はしない。

BWQA_CACHE_DIR="${BWQA_CACHE_DIR:-$HOME/.cache/bw-quickaccess}"
BWQA_SESSION_ISSUED_AT_FILE="$BWQA_CACHE_DIR/session-issued-at"
BWQA_LAST_ITEM_FILE="$BWQA_CACHE_DIR/last-item-id"
BWQA_ERROR_LOG_FILE="$BWQA_CACHE_DIR/last-error.log"
BWQA_COPY_STATUS_FILE="$BWQA_CACHE_DIR/copy-status"
BWQA_COPY_LOCK_FILE="$BWQA_CACHE_DIR/copy-lock"
BWQA_COPY_SPIN_FRAME_FILE="$BWQA_CACHE_DIR/copy-spin-frame"
BWQA_SESSION_TTL_SECONDS="${BWQA_SESSION_TTL_SECONDS:-900}"
BWQA_KEYCHAIN_SERVICE="bw-quickaccess"
BWQA_KEYCHAIN_ACCOUNT="${USER:-bw-quickaccess}"

bwqa_log() {
  printf 'bw-quickaccess: %s\n' "$*" >&2
}

bwqa_die() {
  # BWQA_MSG_* はこのプロジェクトが定義する固定テンプレート(lib/i18n/*.sh)であり、
  # ユーザー入力ではないため、変数を printf の書式文字列として使う設計を許容する。
  # shellcheck disable=SC2059
  bwqa_log "$(printf "$BWQA_MSG_ERR_PREFIX" "$*")"
  exit 1
}

# 表示言語を判定する。優先順位: BWQA_LANG(明示指定) > LC_ALL > LANG。
# ja で始まらない場合、または対応する言語ファイルが無い場合は en にフォールバックする。
bwqa_detect_lang() {
  local lang="${BWQA_LANG:-}"
  if [[ -z "$lang" ]]; then
    local locale="${LC_ALL:-${LANG:-}}"
    case "$locale" in
      ja*) lang="ja" ;;
      *) lang="en" ;;
    esac
  fi
  case "$lang" in
    ja | en) printf '%s' "$lang" ;;
    *) printf '%s' "en" ;;
  esac
}

bwqa_ensure_cache_dir() {
  if [[ ! -d "$BWQA_CACHE_DIR" ]]; then
    mkdir -p "$BWQA_CACHE_DIR"
  fi
  chmod 700 "$BWQA_CACHE_DIR"
}

# 2つのバージョン文字列 "a.b.c" を比較する(v1 >= v2 なら真)
bwqa_version_ge() {
  local v1="$1" v2="$2"
  local -a a b
  IFS='.' read -r -a a <<<"$v1"
  IFS='.' read -r -a b <<<"$v2"
  local i ai bi
  for i in 0 1 2; do
    ai="${a[i]:-0}"
    bi="${b[i]:-0}"
    ai="${ai//[!0-9]/}"
    bi="${bi//[!0-9]/}"
    ai="${ai:-0}"
    bi="${bi:-0}"
    if ((ai > bi)); then
      return 0
    fi
    if ((ai < bi)); then
      return 1
    fi
  done
  return 0
}

# === i18n-load:begin ===
# script/build.sh はバンドル生成時にこのブロックを、lib/i18n/*.sh の中身を
# 埋め込んだ静的な case 文に置き換える(単一自己完結ファイルにするため)。
# 開発時(lib/*.sh をリポジトリ内から直接 source する場合)はここでファイルから動的に読み込む。
BWQA_LIB_DIR_INTERNAL="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BWQA_LANG_RESOLVED="$(bwqa_detect_lang)"
if [[ ! -f "$BWQA_LIB_DIR_INTERNAL/i18n/${BWQA_LANG_RESOLVED}.sh" ]]; then
  BWQA_LANG_RESOLVED="en"
fi
# shellcheck disable=SC1090
source "$BWQA_LIB_DIR_INTERNAL/i18n/${BWQA_LANG_RESOLVED}.sh"
# === i18n-load:end ===

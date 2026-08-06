# bw-quickaccess: 共有パス・ログ・キャッシュディレクトリ管理
# このファイルは source される前提で、直接実行はしない。

BWQA_CACHE_DIR="${BWQA_CACHE_DIR:-$HOME/.cache/bw-quickaccess}"
BWQA_SESSION_ISSUED_AT_FILE="$BWQA_CACHE_DIR/session-issued-at"
BWQA_LAST_ITEM_FILE="$BWQA_CACHE_DIR/last-item-id"
BWQA_ERROR_LOG_FILE="$BWQA_CACHE_DIR/last-error.log"
BWQA_SESSION_TTL_SECONDS="${BWQA_SESSION_TTL_SECONDS:-900}"
BWQA_KEYCHAIN_SERVICE="bw-quickaccess"
BWQA_KEYCHAIN_ACCOUNT="${USER:-bw-quickaccess}"

bwqa_log() {
  printf 'bw-quickaccess: %s\n' "$*" >&2
}

bwqa_die() {
  bwqa_log "エラー: $*"
  exit 1
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

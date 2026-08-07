# bw-quickaccess: bw unlock の session token 管理(OS keychain キャッシュ + TTL + 実利用時フォールバック)

BWQA_SESSION=""

bwqa_keychain_store() {
  local token="$1"
  case "$BWQA_OS_KIND" in
    macos)
      security add-generic-password \
        -a "$BWQA_KEYCHAIN_ACCOUNT" \
        -s "$BWQA_KEYCHAIN_SERVICE" \
        -w "$token" \
        -U >/dev/null 2>&1
      ;;
    linux)
      secret-tool store \
        --label="Bitwarden CLI session (bw-quickaccess)" \
        service "$BWQA_KEYCHAIN_SERVICE" \
        account "$BWQA_KEYCHAIN_ACCOUNT" <<<"$token" >/dev/null 2>&1
      ;;
  esac
}

bwqa_keychain_lookup() {
  case "$BWQA_OS_KIND" in
    macos)
      security find-generic-password \
        -a "$BWQA_KEYCHAIN_ACCOUNT" \
        -s "$BWQA_KEYCHAIN_SERVICE" \
        -w 2>/dev/null
      ;;
    linux)
      secret-tool lookup service "$BWQA_KEYCHAIN_SERVICE" account "$BWQA_KEYCHAIN_ACCOUNT" 2>/dev/null
      ;;
  esac
}

bwqa_keychain_delete() {
  case "$BWQA_OS_KIND" in
    macos)
      security delete-generic-password \
        -a "$BWQA_KEYCHAIN_ACCOUNT" \
        -s "$BWQA_KEYCHAIN_SERVICE" >/dev/null 2>&1 || true
      ;;
    linux)
      secret-tool clear service "$BWQA_KEYCHAIN_SERVICE" account "$BWQA_KEYCHAIN_ACCOUNT" >/dev/null 2>&1 || true
      ;;
  esac
}

bwqa_session_ttl_expired() {
  [[ -f "$BWQA_SESSION_ISSUED_AT_FILE" ]] || return 0
  local issued_at now
  issued_at="$(cat "$BWQA_SESSION_ISSUED_AT_FILE" 2>/dev/null || echo 0)"
  [[ "$issued_at" =~ ^[0-9]+$ ]] || issued_at=0
  now="$(date +%s)"
  ((now - issued_at >= BWQA_SESSION_TTL_SECONDS))
}

# マスターパスワードを対話的に入力させ、bw unlock で session token を取得する。
# 取得した token は標準出力に出さず、内部変数・(可能なら)keychain にのみ保持する。
bwqa_unlock() {
  local token
  bwqa_log "$BWQA_MSG_SESSION_UNLOCKING"
  token="$(bw unlock --raw)" || bwqa_die "$BWQA_MSG_SESSION_UNLOCK_FAILED"
  [[ -n "$token" ]] || bwqa_die "$BWQA_MSG_SESSION_EMPTY"

  if [[ "$BWQA_KEYCHAIN_AVAILABLE" == "true" ]]; then
    bwqa_keychain_store "$token"
    date +%s >"$BWQA_SESSION_ISSUED_AT_FILE"
  fi
  BWQA_SESSION="$token"
}

bwqa_get_session() {
  if [[ "$BWQA_KEYCHAIN_AVAILABLE" == "true" ]] && ! bwqa_session_ttl_expired; then
    local cached
    cached="$(bwqa_keychain_lookup || true)"
    if [[ -n "$cached" ]]; then
      BWQA_SESSION="$cached"
      return 0
    fi
  fi
  bwqa_unlock
}

# bw コマンドを $BWQA_SESSION 付きで実行する。session が無効と判定された場合は
# 再度マスターパスワード入力を求めて1回だけリトライする。
bwqa_bw() {
  local out errfile err status
  errfile="$(mktemp)"
  out="$(BW_SESSION="$BWQA_SESSION" bw "$@" 2>"$errfile")"
  status=$?
  err="$(cat "$errfile")"
  rm -f "$errfile"

  if [[ $status -ne 0 ]] && grep -qiE 'not logged in|vault is locked|invalid session|unauthorized|session' <<<"$err"; then
    bwqa_log "$BWQA_MSG_SESSION_REAUTH"
    bwqa_keychain_delete
    rm -f "$BWQA_SESSION_ISSUED_AT_FILE"
    bwqa_unlock

    errfile="$(mktemp)"
    out="$(BW_SESSION="$BWQA_SESSION" bw "$@" 2>"$errfile")"
    status=$?
    err="$(cat "$errfile")"
    rm -f "$errfile"
  fi

  if [[ $status -ne 0 ]]; then
    # shellcheck disable=SC2059
    bwqa_log "$(printf "$BWQA_MSG_SESSION_BW_CMD_FAILED" "$*" "$err")"
    return "$status"
  fi
  printf '%s' "$out"
}

bwqa_cmd_lock() {
  bwqa_keychain_delete
  rm -f "$BWQA_SESSION_ISSUED_AT_FILE"
  bw lock >/dev/null 2>&1 || true
  bwqa_log "$BWQA_MSG_SESSION_CACHE_CLEARED"
}

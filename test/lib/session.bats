#!/usr/bin/env bats
# lib/session.sh のテスト(bwqa_session_ttl_expired のみ。bw unlock/keychain 連携はスコープ外)

load '../helpers/stub'

setup() {
  bwqa_test_stub_setup
  source "$BWQA_LIB_DIR/common.sh"
  source "$BWQA_LIB_DIR/session.sh"
}

teardown() {
  bwqa_test_stub_teardown
}

@test "bwqa_session_ttl_expired: issued-at ファイルが存在しない場合は期限切れ扱い" {
  run bwqa_session_ttl_expired
  [ "$status" -eq 0 ]
}

@test "bwqa_session_ttl_expired: TTL 未満の経過時間では期限内" {
  printf '%s' "$(( $(date +%s) - (BWQA_SESSION_TTL_SECONDS - 1) ))" >"$BWQA_SESSION_ISSUED_AT_FILE"
  run bwqa_session_ttl_expired
  [ "$status" -ne 0 ]
}

@test "bwqa_session_ttl_expired: TTL ちょうどの経過時間では期限切れ(境界値は >=)" {
  printf '%s' "$(( $(date +%s) - BWQA_SESSION_TTL_SECONDS ))" >"$BWQA_SESSION_ISSUED_AT_FILE"
  run bwqa_session_ttl_expired
  [ "$status" -eq 0 ]
}

@test "bwqa_session_ttl_expired: TTL 超過の経過時間では期限切れ" {
  printf '%s' "$(( $(date +%s) - (BWQA_SESSION_TTL_SECONDS + 100) ))" >"$BWQA_SESSION_ISSUED_AT_FILE"
  run bwqa_session_ttl_expired
  [ "$status" -eq 0 ]
}

@test "bwqa_session_ttl_expired: issued-at の内容が数値でない場合は期限切れ扱い" {
  printf 'not-a-number' >"$BWQA_SESSION_ISSUED_AT_FILE"
  run bwqa_session_ttl_expired
  [ "$status" -eq 0 ]
}

@test "bwqa_session_ttl_expired: issued-at の内容が空文字の場合は期限切れ扱い" {
  printf '' >"$BWQA_SESSION_ISSUED_AT_FILE"
  run bwqa_session_ttl_expired
  [ "$status" -eq 0 ]
}

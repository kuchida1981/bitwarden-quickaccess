#!/usr/bin/env bats
# lib/fields.sh のテスト(フィールド選択画面の fzf 対話部分はスコープ外)

load '../helpers/stub'

setup() {
  bwqa_test_stub_setup
  source "$BWQA_LIB_DIR/common.sh"
  source "$BWQA_LIB_DIR/preflight.sh"
  source "$BWQA_LIB_DIR/clipboard.sh"
  source "$BWQA_LIB_DIR/session.sh"
  source "$BWQA_LIB_DIR/fields.sh"

  # bwqa_copy_field_internal は無条件に platform/clipboard 検出を呼ぶため、
  # ホスト環境(特に GUI の無い Linux CI ランナー)に依存しないよう関数スタブで
  # 差し替える。実際に使うクリップボードコマンドは PATH ダミーの clipboard-capture。
  # BWQA_OS_KIND/BWQA_CLIPBOARD_CMD_ARR は lib/fields.sh 側の bwqa_copy_field_internal
  # から間接的に参照される(SC2034/SC2329 は静的解析の誤検知のためインラインで無効化)。
  # shellcheck disable=SC2034
  bwqa_detect_platform() { BWQA_OS_KIND="macos"; }
  # shellcheck disable=SC2034
  bwqa_detect_clipboard_cmd() { BWQA_CLIPBOARD_CMD_ARR=(clipboard-capture); }

  bwqa_test_stub_cmd clipboard-capture 'cat >"$BWQA_TEST_CACHE_DIR/clipboard-output"'
}

teardown() {
  bwqa_test_stub_teardown
}

# --- 5.1 bwqa_build_field_rows -------------------------------------------

@test "bwqa_build_field_rows: password/username/totp すべてある場合、username を先頭行にする" {
  local summary='{"has_password":true,"has_username":true,"has_totp":true}'
  run bwqa_build_field_rows "$summary"
  [ "$status" -eq 0 ]

  local first_field
  first_field="$(echo "$output" | head -n1 | cut -f1)"
  [ "$first_field" = "username" ]

  local line_count
  line_count="$(echo "$output" | wc -l | tr -d ' ')"
  [ "$line_count" -eq 3 ]
}

@test "bwqa_build_field_rows: username が無く、password と totp のみがあるアイテムでは、password → totp の順で出力される" {
  local summary='{"has_password":true,"has_username":false,"has_totp":true}'
  run bwqa_build_field_rows "$summary"
  [ "$status" -eq 0 ]

  local first_field
  first_field="$(echo "$output" | head -n1 | cut -f1)"
  [ "$first_field" = "password" ]

  local second_field
  second_field="$(echo "$output" | sed -n '2p' | cut -f1)"
  [ "$second_field" = "totp" ]

  local line_count
  line_count="$(echo "$output" | wc -l | tr -d ' ')"
  [ "$line_count" -eq 2 ]
}

@test "bwqa_build_field_rows: password が無い場合は password 行を出力しない" {
  local summary='{"has_password":false,"has_username":true,"has_totp":false}'
  run bwqa_build_field_rows "$summary"
  [ "$status" -eq 0 ]
  [ "$output" = "$(printf 'username\tユーザー名をコピー (ctrl-u)')" ]
}

@test "bwqa_build_field_rows: コピー可能なフィールドが無い場合は空行を出力する" {
  local summary='{"has_password":false,"has_username":false,"has_totp":false}'
  run bwqa_build_field_rows "$summary"
  [ "$status" -eq 0 ]
  [ -z "$output" ]
}

# --- 5.2 bwqa_get_item_summary --------------------------------------------

@test "bwqa_get_item_summary: 全フィールドありのアイテムを正しく整形する" {
  bwqa_bw() { jq -c '.all_fields' "$BWQA_TEST_FIXTURES_DIR/bw-get-item.json"; }

  local output status
  output="$(bwqa_get_item_summary "11111111-1111-1111-1111-111111111111" 2>/dev/null)"
  status=$?
  [ "$status" -eq 0 ]

  local summary
  summary="$output"
  [ "$(echo "$summary" | jq -r '.name')" = "GitHub" ]
  [ "$(echo "$summary" | jq -r '.has_username')" = "true" ]
  [ "$(echo "$summary" | jq -r '.has_password')" = "true" ]
  [ "$(echo "$summary" | jq -r '.has_totp')" = "true" ]
}

@test "bwqa_get_item_summary: フィールドが無いアイテムは has_* がすべて false になる" {
  bwqa_bw() { jq -c '.no_fields' "$BWQA_TEST_FIXTURES_DIR/bw-get-item.json"; }

  local output status
  output="$(bwqa_get_item_summary "55555555-5555-5555-5555-555555555555" 2>/dev/null)"
  status=$?
  [ "$status" -eq 0 ]

  local summary
  summary="$output"
  [ "$(echo "$summary" | jq -r '.has_username')" = "false" ]
  [ "$(echo "$summary" | jq -r '.has_password')" = "false" ]
  [ "$(echo "$summary" | jq -r '.has_totp')" = "false" ]
}

# --- 5.3 bwqa_copy_field_internal: 正常系 ----------------------------------

# password/username/totp すべてに値を返す bw スタブ。5.3 の複数テストで共用する。
_stub_bw_get_all_fields() {
  bwqa_test_stub_cmd bw '
case "$2" in
  password) printf "correct horse battery staple\n" ;;
  username) printf "alice@example.com\n" ;;
  totp) printf "123456\n" ;;
  *) exit 1 ;;
esac
'
}

@test "bwqa_copy_field_internal: password 取得成功時にクリップボードへコピーする" {
  _stub_bw_get_all_fields

  BWQA_ITEM_ID="11111111-1111-1111-1111-111111111111" BW_SESSION="dummy-session" \
    run bwqa_copy_field_internal password
  [ "$status" -eq 0 ]
  [ "$(cat "$BWQA_TEST_CACHE_DIR/clipboard-output")" = "correct horse battery staple" ]
}

@test "bwqa_copy_field_internal: username/totp もそれぞれ正しくコピーする" {
  _stub_bw_get_all_fields

  BWQA_ITEM_ID="11111111-1111-1111-1111-111111111111" BW_SESSION="dummy-session" \
    run bwqa_copy_field_internal username
  [ "$status" -eq 0 ]
  [ "$(cat "$BWQA_TEST_CACHE_DIR/clipboard-output")" = "alice@example.com" ]

  BWQA_ITEM_ID="11111111-1111-1111-1111-111111111111" BW_SESSION="dummy-session" \
    run bwqa_copy_field_internal totp
  [ "$status" -eq 0 ]
  [ "$(cat "$BWQA_TEST_CACHE_DIR/clipboard-output")" = "123456" ]
}

# --- 5.4 bwqa_copy_field_internal: 異常系 -----------------------------------

@test "bwqa_copy_field_internal: 値取得結果が空の場合はエラーログに記録して終了する" {
  bwqa_test_stub_cmd bw 'printf ""'

  BWQA_ITEM_ID="11111111-1111-1111-1111-111111111111" BW_SESSION="dummy-session" \
    run bwqa_copy_field_internal password
  [ "$status" -ne 0 ]

  run grep -q "field=password" "$BWQA_ERROR_LOG_FILE"
  [ "$status" -eq 0 ]
  run grep -q "11111111-1111-1111-1111-111111111111" "$BWQA_ERROR_LOG_FILE"
  [ "$status" -eq 0 ]
}

@test "bwqa_copy_field_internal: 不明な field 名を指定した場合はエラーログに記録して終了する" {
  bwqa_test_stub_cmd bw 'printf "should-not-be-called\n"'

  BWQA_ITEM_ID="11111111-1111-1111-1111-111111111111" BW_SESSION="dummy-session" \
    run bwqa_copy_field_internal not-a-real-field
  [ "$status" -ne 0 ]

  run grep -q "不明な field です: not-a-real-field" "$BWQA_ERROR_LOG_FILE"
  [ "$status" -eq 0 ]
}

@test "bwqa_copy_field_internal: item_id/session が無い場合はエラーログに記録して終了する" {
  run bwqa_copy_field_internal password
  [ "$status" -ne 0 ]

  run grep -q "item_id/session/field のいずれかが不足しています" "$BWQA_ERROR_LOG_FILE"
  [ "$status" -eq 0 ]
}

@test "bwqa_get_item_summary: ローディングメッセージが stderr に出力されること" {
  bwqa_bw() { jq -c '.all_fields' "$BWQA_TEST_FIXTURES_DIR/bw-get-item.json"; }

  local err
  err="$(bwqa_get_item_summary "11111111-1111-1111-1111-111111111111" 2>&1 1>/dev/null)"
  [[ "$err" == *"アイテム情報を取得しています..."* ]]
}

@test "bwqa_copy_field_internal: コピー成功時にステータスファイルに成功メッセージが書き込まれること" {
  _stub_bw_get_all_fields

  BWQA_ITEM_ID="11111111-1111-1111-1111-111111111111" BW_SESSION="dummy-session" \
    run bwqa_copy_field_internal password
  [ "$status" -eq 0 ]
  [ "$(cat "$BWQA_COPY_STATUS_FILE")" = "パスワードをコピーしました" ]
}

@test "bwqa_copy_field_internal: 値取得結果が空の場合にステータスファイルに未設定メッセージが書き込まれること" {
  bwqa_test_stub_cmd bw 'printf ""'

  BWQA_ITEM_ID="11111111-1111-1111-1111-111111111111" BW_SESSION="dummy-session" \
    run bwqa_copy_field_internal password
  [ "$status" -ne 0 ]
  [ "$(cat "$BWQA_COPY_STATUS_FILE")" = "パスワードは設定されていません" ]
}

@test "bwqa_copy_field_internal: bwコマンドが失敗した場合にステータスファイルに失敗メッセージが書き込まれること" {
  bwqa_test_stub_cmd bw 'exit 1'

  BWQA_ITEM_ID="11111111-1111-1111-1111-111111111111" BW_SESSION="dummy-session" \
    run bwqa_copy_field_internal password
  [ "$status" -ne 0 ]
  [ "$(cat "$BWQA_COPY_STATUS_FILE")" = "コピーに失敗しました" ]
}

@test "bwqa_copy_field_internal: クリップボードコマンド自体が失敗した場合もステータスファイルに失敗メッセージが書き込まれること" {
  _stub_bw_get_all_fields
  bwqa_test_stub_cmd clipboard-capture 'exit 1'

  BWQA_ITEM_ID="11111111-1111-1111-1111-111111111111" BW_SESSION="dummy-session" \
    run bwqa_copy_field_internal password
  [ "$status" -ne 0 ]
  [ "$(cat "$BWQA_COPY_STATUS_FILE")" = "コピーに失敗しました" ]
}


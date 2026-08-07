#!/usr/bin/env bats
# lib/search.sh のテスト(bwqa_fetch_items のみ。fzf 対話画面はスコープ外)

load '../helpers/stub'

setup() {
  bwqa_test_stub_setup
  source "$BWQA_LIB_DIR/common.sh"
  source "$BWQA_LIB_DIR/session.sh"
  source "$BWQA_LIB_DIR/search.sh"

  # lib/search.sh の bwqa_fetch_items() から間接的に呼ばれる
  # (SC2329 は .shellcheckrc でグローバルに無効化済み)。
  bwqa_bw() {
    cat "$BWQA_TEST_FIXTURES_DIR/bw-list-items.json"
  }
}

teardown() {
  bwqa_test_stub_teardown
}

@test "bwqa_fetch_items: type==1 のアイテムのみに絞り込む" {
  local output status
  output="$(bwqa_fetch_items 2>/dev/null)"
  status=$?
  [ "$status" -eq 0 ]

  local count
  count="$(echo "$output" | jq 'length')"
  [ "$count" -eq 3 ]

  local ids
  ids="$(echo "$output" | jq -r '.[].id' | sort)"
  [ "$ids" = "$(printf '11111111-1111-1111-1111-111111111111\n22222222-2222-2222-2222-222222222222\n33333333-3333-3333-3333-333333333333')" ]
}

@test "bwqa_fetch_items: 各エントリは id/label フィールドを持つ" {
  local output status
  output="$(bwqa_fetch_items 2>/dev/null)"
  status=$?
  [ "$status" -eq 0 ]

  local keys
  keys="$(echo "$output" | jq -r '.[0] | keys | sort | join(",")')"
  [ "$keys" = "id,label" ]
}

@test "bwqa_fetch_items: username がある場合は label に括弧付きで付与する" {
  local output status
  output="$(bwqa_fetch_items 2>/dev/null)"
  status=$?
  [ "$status" -eq 0 ]

  local label
  label="$(echo "$output" | jq -r '.[] | select(.id == "11111111-1111-1111-1111-111111111111") | .label')"
  [ "$label" = "GitHub (alice@example.com)" ]
}

@test "bwqa_fetch_items: username が無い場合は label に括弧を付与しない" {
  local output status
  output="$(bwqa_fetch_items 2>/dev/null)"
  status=$?
  [ "$status" -eq 0 ]

  local label
  label="$(echo "$output" | jq -r '.[] | select(.id == "22222222-2222-2222-2222-222222222222") | .label')"
  [ "$label" = "No Username Service" ]
}

@test "bwqa_fetch_items: name に含まれるタブ/改行/CR をスペースに正規化する" {
  local output status
  output="$(bwqa_fetch_items 2>/dev/null)"
  status=$?
  [ "$status" -eq 0 ]

  local label
  label="$(echo "$output" | jq -r '.[] | select(.id == "33333333-3333-3333-3333-333333333333") | .label')"
  [ "$label" = "Weird Name With Control Chars (bob@example.com)" ]

  # 制御文字が残っていないことも確認する
  [[ "$label" != *$'\t'* ]]
  [[ "$label" != *$'\n'* ]]
  [[ "$label" != *$'\r'* ]]
}

@test "bwqa_fetch_items: vault読み込み中のメッセージが stderr に出力されること" {
  local err
  err="$(bwqa_fetch_items 2>&1 1>/dev/null)"
  [[ "$err" == *"vaultのアイテム一覧を読み込んでいます..."* ]]
}

# --- 2 bwqa_run_search_screen: fzf 起動オプション ---

@test "bwqa_run_search_screen: --height オプションを付けずに fzf を起動する(フルスクリーン化)" {
  bwqa_test_stub_cmd fzf 'printf "%s\n" "$@" >"$BWQA_TEST_CACHE_DIR/fzf-args"'
  BWQA_SELF="/tmp/bwqa-self-stub" BWQA_SESSION="dummy-session" \
    run bwqa_run_search_screen
  [ "$status" -eq 0 ]

  run grep -q "--height" "$BWQA_TEST_CACHE_DIR/fzf-args"
  [ "$status" -ne 0 ]
}

@test "bwqa_run_search_screen: every(0.15):bg-transform-border-label バインドで __copy-status を呼ぶ" {
  bwqa_test_stub_cmd fzf 'printf "%s\n" "$@" >"$BWQA_TEST_CACHE_DIR/fzf-args"'
  BWQA_SELF="/tmp/bwqa-self-stub" BWQA_SESSION="dummy-session" \
    run bwqa_run_search_screen
  [ "$status" -eq 0 ]

  run grep -q "every(0.15):bg-transform-border-label" "$BWQA_TEST_CACHE_DIR/fzf-args"
  [ "$status" -eq 0 ]
  run grep -q "__copy-status" "$BWQA_TEST_CACHE_DIR/fzf-args"
  [ "$status" -eq 0 ]
}

@test "bwqa_run_search_screen: ctrl-o/ctrl-r/ctrl-t のコピー処理はバックグラウンドジョブとして起動する" {
  bwqa_test_stub_cmd fzf 'printf "%s\n" "$@" >"$BWQA_TEST_CACHE_DIR/fzf-args"'
  BWQA_SELF="/tmp/bwqa-self-stub" BWQA_SESSION="dummy-session" \
    run bwqa_run_search_screen
  [ "$status" -eq 0 ]

  run grep -q "__copy-field username &" "$BWQA_TEST_CACHE_DIR/fzf-args"
  [ "$status" -eq 0 ]
  run grep -q "__copy-field password &" "$BWQA_TEST_CACHE_DIR/fzf-args"
  [ "$status" -eq 0 ]
  run grep -q "__copy-field totp &" "$BWQA_TEST_CACHE_DIR/fzf-args"
  [ "$status" -eq 0 ]
}

@test "bwqa_run_search_screen: ロックファイルはバックグラウンド化する前に同期的に作成する(スピナー表示の競合防止)" {
  bwqa_test_stub_cmd fzf 'printf "%s\n" "$@" >"$BWQA_TEST_CACHE_DIR/fzf-args"'
  BWQA_SELF="/tmp/bwqa-self-stub" BWQA_SESSION="dummy-session" \
    run bwqa_run_search_screen
  [ "$status" -eq 0 ]

  # execute-silent 内で `: >"$BWQA_COPY_LOCK_FILE"` が __copy-field の起動より
  # 先(同じ行の左側)に書かれていることを確認する。__copy-status 側の
  # transform-border-label が execute-silent 完了直後に走っても、この時点で
  # 既にロックファイルが存在することを保証するための並び順。
  run grep -q ": >\"$BWQA_COPY_LOCK_FILE\"; BWQA_ITEM_ID={1} \"/tmp/bwqa-self-stub\" __copy-field password &" "$BWQA_TEST_CACHE_DIR/fzf-args"
  [ "$status" -eq 0 ]
}

# bw-quickaccess: vault アイテムのインクリメンタルサーチ + 直前アイテムキャッシュ

# ログインタイプ(type == 1)のアイテムのみを対象に、fzf 向けの id/label ペアを作る
bwqa_fetch_items() {
  local raw
  bwqa_log "vaultのアイテム一覧を読み込んでいます..."
  raw="$(bwqa_bw list items)" || return 1
  jq -c '
    [.[] | select(.type == 1)]
    | map({
        id,
        label: (
          ((.name // "") + (if ((.login.username // "") != "") then " (" + .login.username + ")" else "" end))
          | gsub("[\t\n\r]"; " ")
        )
      })
  ' <<<"$raw"
}

bwqa_run_search_screen() {
  local items_json
  items_json="$(bwqa_fetch_items)" || bwqa_die "vault アイテムの取得に失敗しました。"

  local status_feedback
  status_feedback="+transform-border-label(cat \"$BWQA_COPY_STATUS_FILE\" 2>/dev/null)"

  local selected_id
  # この subshell 内での export は fzf の execute-silent 経由で起動する
  # __copy-field 子プロセスへ BW_SESSION を継承させるためのもので、subshell の
  # 外に値を戻す意図はない(SC2030/SC2031 は意図した挙動への誤検知)。
  # shellcheck disable=SC2030,SC2031,SC2153
  selected_id="$(
    export BW_SESSION="$BWQA_SESSION"
    jq -r '.[] | [.id, .label] | @tsv' <<<"$items_json" \
      | fzf --delimiter='\t' --with-nth=2 \
        --prompt='vault> ' --height=80% --reverse \
        --header='Enter: アイテムを選択  ctrl-u: ユーザー名  ctrl-p: パスワード  ctrl-t: TOTP を直接コピー  Esc: 終了' \
        --border=rounded --border-label='' \
        --bind="ctrl-u:execute-silent(BWQA_ITEM_ID={1} \"$BWQA_SELF\" __copy-field username)${status_feedback}" \
        --bind="ctrl-p:execute-silent(BWQA_ITEM_ID={1} \"$BWQA_SELF\" __copy-field password)${status_feedback}" \
        --bind="ctrl-t:execute-silent(BWQA_ITEM_ID={1} \"$BWQA_SELF\" __copy-field totp)${status_feedback}" \
      | cut -f1
  )" || true

  printf '%s' "$selected_id"
}

bwqa_read_last_item() {
  [[ -f "$BWQA_LAST_ITEM_FILE" ]] && cat "$BWQA_LAST_ITEM_FILE"
  return 0
}

bwqa_write_last_item() {
  printf '%s' "$1" >"$BWQA_LAST_ITEM_FILE"
}

bwqa_clear_last_item() {
  rm -f "$BWQA_LAST_ITEM_FILE"
}

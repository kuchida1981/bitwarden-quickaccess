# bw-quickaccess: vault アイテムのインクリメンタルサーチ + 直前アイテムキャッシュ

# ログインタイプ(type == 1)のアイテムのみを対象に、fzf 向けの id/label ペアを作る
bwqa_fetch_items() {
  local raw
  bwqa_log "$BWQA_MSG_SEARCH_LOADING_ITEMS"
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
  items_json="$(bwqa_fetch_items)" || bwqa_die "$BWQA_MSG_SEARCH_FETCH_FAILED"

  local status_feedback
  # transform-border-label 側にも {1} を参照させ、execute-silent のコピー処理と
  # 同じ「0件マッチ時は該当 bind action 全体をスキップする」という fzf の挙動を
  # 適用させる。{1} がないと、0件時に execute-silent(コピー本体)だけがスキップされ
  # transform-border-label だけが実行されてしまい、直前のコピー結果メッセージが
  # 「たった今コピーしたかのように」再表示される不具合になる(design.md が想定する
  # 「実質無反応」という前提を満たせない)。`: {1}` は no-op で処理結果には影響しない。
  # __copy-status サブコマンドは、コピー処理中はスピナーを、完了後は
  # BWQA_COPY_STATUS_FILE の内容を返す(lib/fields.sh の bwqa_render_copy_status 参照)。
  status_feedback="+transform-border-label(\"$BWQA_SELF\" __copy-status; : {1})"

  local selected_id
  # この subshell 内での export は fzf の execute-silent 経由で起動する
  # __copy-field 子プロセスへ BW_SESSION を継承させるためのもので、subshell の
  # 外に値を戻す意図はない(SC2030/SC2031 は意図した挙動への誤検知)。
  # shellcheck disable=SC2030,SC2031,SC2153
  selected_id="$(
    export BW_SESSION="$BWQA_SESSION"
    jq -r '.[] | [.id, .label] | @tsv' <<<"$items_json" \
      | fzf --delimiter='\t' --with-nth=2 \
        --prompt='vault> ' --reverse \
        --header="$BWQA_MSG_SEARCH_FZF_HEADER" \
        --border=rounded --border-label='' \
        --bind="ctrl-o:execute-silent(BWQA_ITEM_ID={1} \"$BWQA_SELF\" __copy-field username &)${status_feedback}" \
        --bind="ctrl-r:execute-silent(BWQA_ITEM_ID={1} \"$BWQA_SELF\" __copy-field password &)${status_feedback}" \
        --bind="ctrl-t:execute-silent(BWQA_ITEM_ID={1} \"$BWQA_SELF\" __copy-field totp &)${status_feedback}" \
        --bind="every(0.15):bg-transform-border-label(\"$BWQA_SELF\" __copy-status)" \
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

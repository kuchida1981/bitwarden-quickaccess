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

  local selected_id
  selected_id="$(
    jq -r '.[] | [.id, .label] | @tsv' <<<"$items_json" \
      | fzf --delimiter='\t' --with-nth=2 \
        --prompt='vault> ' --height=80% --reverse \
        --header='Enter: アイテムを選択  Esc: 終了' \
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

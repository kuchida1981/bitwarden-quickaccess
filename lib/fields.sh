# bw-quickaccess: フィールド選択画面(ユーザー名/パスワード/TOTP)とコピー処理
#
# フィールド選択画面は fzf の --bind execute-silent を使い、コピーしても画面を
# 閉じずに連続コピーできるようにする。実際のコピー処理は、この同じスクリプトを
# `__copy-field` サブコマンドとして再帰的に起動することで実行する(fzf の
# execute-silent はシェル経由でコマンドを実行するため)。
# アイテム ID / session token はコマンド文字列に埋め込まず、環境変数
# (BWQA_ITEM_ID / BW_SESSION)経由で子プロセスに継承させる。

bwqa_get_item_summary() {
  local item_id="$1" raw
  raw="$(bwqa_bw get item "$item_id")" || return 1
  jq -c '{
    name: (.name // ""),
    has_username: ((.login.username // "") != ""),
    has_password: ((.login.password // "") != ""),
    has_totp: ((.login.totp // "") != "")
  }' <<<"$raw"
}

# password を先頭行にして、Enter(先頭行選択)がパスワードコピーになりやすくする
bwqa_build_field_rows() {
  local summary_json="$1"
  jq -r '
    [
      (if .has_password then ["password", "パスワードをコピー (ctrl-p)"] else empty end),
      (if .has_username then ["username", "ユーザー名をコピー (ctrl-u)"] else empty end),
      (if .has_totp then ["totp", "TOTP をコピー (ctrl-t)"] else empty end)
    ] | .[] | @tsv
  ' <<<"$summary_json"
}

# 戻り値:
#   0 = 明示終了(q)
#   1 = Esc(検索画面へ戻る)
#   2 = コピー可能なフィールドがない
bwqa_run_field_screen() {
  local item_id="$1"
  local summary
  summary="$(bwqa_get_item_summary "$item_id")" || bwqa_die "アイテム情報の取得に失敗しました。"

  local item_name
  item_name="$(jq -r '.name' <<<"$summary")"

  local rows
  rows="$(bwqa_build_field_rows "$summary")"
  if [[ -z "$rows" ]]; then
    bwqa_log "コピー可能なフィールドがありません: ${item_name}"
    return 2
  fi

  local key
  key="$(
    export BW_SESSION="$BWQA_SESSION"
    export BWQA_ITEM_ID="$item_id"
    printf '%s\n' "$rows" | fzf \
      --delimiter='\t' --with-nth=2 \
      --prompt="${item_name} > " --height=80% --reverse \
      --header='Enter: 選択中の項目をコピー  ctrl-p: password  ctrl-u: username  ctrl-t: totp  Esc: 検索へ戻る  q: 終了' \
      --expect='esc,q' \
      --bind="enter:execute-silent(\"$BWQA_SELF\" __copy-field {1})" \
      --bind="ctrl-p:execute-silent(\"$BWQA_SELF\" __copy-field password)" \
      --bind="ctrl-u:execute-silent(\"$BWQA_SELF\" __copy-field username)" \
      --bind="ctrl-t:execute-silent(\"$BWQA_SELF\" __copy-field totp)" \
      | head -n1
  )" || true

  case "$key" in
    esc) return 1 ;;
    *) return 0 ;;
  esac
}

# __copy-field サブコマンドの実体。BWQA_ITEM_ID / BW_SESSION は環境変数から受け取る。
# 機密情報は標準出力へは一切出さず、クリップボードへのみ渡す。失敗はログファイルにのみ記録する。
bwqa_copy_field_internal() {
  local field="${1:-}"
  local item_id="${BWQA_ITEM_ID:-}"
  local session="${BW_SESSION:-}"

  bwqa_ensure_cache_dir
  bwqa_detect_platform
  bwqa_detect_clipboard_cmd

  if [[ -z "$item_id" || -z "$session" || -z "$field" ]]; then
    printf '%s __copy-field: item_id/session/field のいずれかが不足しています\n' "$(date '+%F %T')" >>"$BWQA_ERROR_LOG_FILE"
    exit 1
  fi

  local value=""
  case "$field" in
    username) value="$(BW_SESSION="$session" bw get username "$item_id" 2>>"$BWQA_ERROR_LOG_FILE")" || true ;;
    password) value="$(BW_SESSION="$session" bw get password "$item_id" 2>>"$BWQA_ERROR_LOG_FILE")" || true ;;
    totp) value="$(BW_SESSION="$session" bw get totp "$item_id" 2>>"$BWQA_ERROR_LOG_FILE")" || true ;;
    *)
      printf '%s __copy-field: 不明な field です: %s\n' "$(date '+%F %T')" "$field" >>"$BWQA_ERROR_LOG_FILE"
      exit 1
      ;;
  esac

  if [[ -z "$value" ]]; then
    printf '%s __copy-field: field=%s item=%s の値が空でした\n' "$(date '+%F %T')" "$field" "$item_id" >>"$BWQA_ERROR_LOG_FILE"
    exit 1
  fi

  printf '%s' "$value" | bwqa_copy_to_clipboard
}

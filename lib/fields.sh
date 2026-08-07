# bw-quickaccess: フィールド選択画面(ユーザー名/パスワード/TOTP)とコピー処理
#
# フィールド選択画面は fzf の --bind execute-silent を使い、コピーしても画面を
# 閉じずに連続コピーできるようにする。実際のコピー処理は、この同じスクリプトを
# `__copy-field` サブコマンドとして再帰的に起動することで実行する(fzf の
# execute-silent はシェル経由でコマンドを実行するため)。
# session token(実際の認証情報)はコマンド文字列に埋め込まず、環境変数
# (BW_SESSION)経由で子プロセスに継承させる。アイテム ID は vault アイテムの
# UUID に過ぎず、それ単体では何の情報も取得できない非秘匿な識別子であるため、
# fzf の `{1}` プレースホルダでコマンド文字列に埋め込むことを許容する
# (このファイルの enter バインドではフィールド名を、lib/search.sh の
# ctrl-o/r、ctrl-t バインドではアイテム ID を、それぞれ {1} で埋め込んでいる)。

bwqa_get_item_summary() {
  local item_id="$1" raw
  bwqa_log "$BWQA_MSG_FIELDS_LOADING_ITEM"
  raw="$(bwqa_bw get item "$item_id")" || return 1
  jq -c '{
    name: (.name // ""),
    has_username: ((.login.username // "") != ""),
    has_password: ((.login.password // "") != ""),
    has_totp: ((.login.totp // "") != "")
  }' <<<"$raw"
}

# username を先頭にすることで、Enterによる誤ったパスワードコピーの事故を減らす
bwqa_build_field_rows() {
  local summary_json="$1"
  jq -r \
    --arg u "$BWQA_MSG_FIELDS_ROW_USERNAME" \
    --arg p "$BWQA_MSG_FIELDS_ROW_PASSWORD" \
    --arg t "$BWQA_MSG_FIELDS_ROW_TOTP" '
    [
      (if .has_username then ["username", $u] else empty end),
      (if .has_password then ["password", $p] else empty end),
      (if .has_totp then ["totp", $t] else empty end)
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
  summary="$(bwqa_get_item_summary "$item_id")" || bwqa_die "$BWQA_MSG_FIELDS_ITEM_FETCH_FAILED"

  local item_name
  item_name="$(jq -r '.name' <<<"$summary")"

  local rows
  rows="$(bwqa_build_field_rows "$summary")"
  if [[ -z "$rows" ]]; then
    # BWQA_MSG_* はこのプロジェクトが定義する固定テンプレート(lib/i18n/*.sh)であり、
    # ユーザー入力ではないため、変数を printf の書式文字列として使う設計を許容する。
    # shellcheck disable=SC2059
    bwqa_log "$(printf "$BWQA_MSG_FIELDS_NO_COPYABLE_FIELDS" "$item_name")"
    return 2
  fi

  local key
  local status_feedback
  status_feedback="+transform-border-label(cat \"$BWQA_COPY_STATUS_FILE\" 2>/dev/null)"
  # この subshell 内での export は fzf の execute-silent 経由で起動する
  # __copy-field 子プロセスへ環境変数を継承させるためのもので、subshell の
  # 外に値を戻す意図はない(SC2030/SC2031 は意図した挙動への誤検知)。
  # shellcheck disable=SC2030,SC2031,SC2153
  key="$(
    export BW_SESSION="$BWQA_SESSION"
    export BWQA_ITEM_ID="$item_id"
    printf '%s\n' "$rows" | fzf \
      --delimiter='\t' --with-nth=2 \
      --prompt="${item_name} > " --height=80% --reverse \
      --header="$BWQA_MSG_FIELDS_FZF_HEADER" \
      --border=rounded --border-label='' \
      --expect='esc,q' \
      --bind="enter:execute-silent(\"$BWQA_SELF\" __copy-field {1})${status_feedback}" \
      --bind="ctrl-r:execute-silent(\"$BWQA_SELF\" __copy-field password)${status_feedback}" \
      --bind="ctrl-o:execute-silent(\"$BWQA_SELF\" __copy-field username)${status_feedback}" \
      --bind="ctrl-t:execute-silent(\"$BWQA_SELF\" __copy-field totp)${status_feedback}" \
      | head -n1
  )" || true

  case "$key" in
    esc) return 1 ;;
    *) return 0 ;;
  esac
}

bwqa_field_label() {
  case "$1" in
    username) printf '%s' "$BWQA_MSG_FIELDS_LABEL_USERNAME" ;;
    password) printf '%s' "$BWQA_MSG_FIELDS_LABEL_PASSWORD" ;;
    totp) printf '%s' "$BWQA_MSG_FIELDS_LABEL_TOTP" ;;
    *) printf '%s' "$1" ;;
  esac
}

# __copy-field サブコマンドの実体。BWQA_ITEM_ID / BW_SESSION は環境変数から受け取る。
# 機密情報は標準出力へは一切出さず、クリップボードへのみ渡す。失敗はログファイルにのみ記録する。
# この関数は bwqa_run_field_screen 内の subshell とは別の(fzf 経由で再起動される)
# プロセスとして実行されるため、値は subshell 内での export ではなく実際の環境変数
# 経由で渡ってくる(SC2031 は静的解析上の誤検知)。
#
# コピー結果(成功/フィールド未設定/bw コマンド失敗)は BWQA_COPY_STATUS_FILE に1行
# 上書きで書き込む。fzf の execute-silent は子プロセスの標準出力/標準エラー出力を
# 一切ターミナルに表示しないため、fzf 側は別バインド(transform-border-label)経由で
# このファイルを読み出し、画面上にフィードバックとして表示する。
# shellcheck disable=SC2031
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
  local exit_code=0
  # この関数は fzf の execute-silent 経由でのみ呼ばれ、execute-silent は子プロセスの
  # stdout/stderr を一切ターミナルに表示しない(fzf(1) COMMAND EXECUTION 参照)ため、
  # ここで bwqa_log を呼んでもユーザーには見えない。ローディング表示が必要な場合は
  # fzf のヘッダーを書き換える仕組み(別 change で検討)が必要になる。
  case "$field" in
    username) value="$(BW_SESSION="$session" bw get username "$item_id" 2>>"$BWQA_ERROR_LOG_FILE")" || exit_code=$? ;;
    password) value="$(BW_SESSION="$session" bw get password "$item_id" 2>>"$BWQA_ERROR_LOG_FILE")" || exit_code=$? ;;
    totp) value="$(BW_SESSION="$session" bw get totp "$item_id" 2>>"$BWQA_ERROR_LOG_FILE")" || exit_code=$? ;;
    *)
      printf '%s __copy-field: 不明な field です: %s\n' "$(date '+%F %T')" "$field" >>"$BWQA_ERROR_LOG_FILE"
      printf '%s\n' "$BWQA_MSG_FIELDS_COPY_FAILED" >"$BWQA_COPY_STATUS_FILE"
      exit 1
      ;;
  esac

  if [[ $exit_code -ne 0 ]]; then
    printf '%s\n' "$BWQA_MSG_FIELDS_COPY_FAILED" >"$BWQA_COPY_STATUS_FILE"
    exit 1
  fi

  local label
  label="$(bwqa_field_label "$field")"

  if [[ -z "$value" ]]; then
    # shellcheck disable=SC2059
    printf "${BWQA_MSG_FIELDS_VALUE_NOT_SET}\n" "$label" >"$BWQA_COPY_STATUS_FILE"
    printf '%s __copy-field: field=%s item=%s の値が空でした\n' "$(date '+%F %T')" "$field" "$item_id" >>"$BWQA_ERROR_LOG_FILE"
    exit 1
  fi

  if ! printf '%s' "$value" | bwqa_copy_to_clipboard; then
    printf '%s\n' "$BWQA_MSG_FIELDS_COPY_FAILED" >"$BWQA_COPY_STATUS_FILE"
    exit 1
  fi

  # shellcheck disable=SC2059
  printf "${BWQA_MSG_FIELDS_COPY_SUCCESS}\n" "$label" >"$BWQA_COPY_STATUS_FILE"
}

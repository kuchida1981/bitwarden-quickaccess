# bw-quickaccess: クリップボードコピー
# 値は必ず標準入力経由で渡し、コマンドライン引数には載せないこと。

bwqa_copy_to_clipboard() {
  "${BWQA_CLIPBOARD_CMD_ARR[@]}" >/dev/null 2>&1
}

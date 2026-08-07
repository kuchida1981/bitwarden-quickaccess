#!/usr/bin/env bash
# script/build.sh
# bin/bw-quickaccess と lib/*.sh を連結し、単一の自己完結した実行可能スクリプトを生成する。

set -euo pipefail

# スクリプトがあるディレクトリの親（リポジトリのルート）を取得
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# 出力先の設定（引数で上書き可能。デフォルトは dist/bw-quickaccess）
OUTPUT_PATH="${1:-$REPO_ROOT/dist/bw-quickaccess}"

# 出力先ディレクトリの作成
mkdir -p "$(dirname "$OUTPUT_PATH")"

# テンポラリファイルを用意して、そこに構築していく
TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

LIBS=(
  "lib/common.sh"
  "lib/preflight.sh"
  "lib/clipboard.sh"
  "lib/session.sh"
  "lib/search.sh"
  "lib/fields.sh"
)

# lib/common.sh 内の i18n-load マーカー区間を、lib/i18n/*.sh の中身を埋め込んだ
# 静的な case 文に置き換えて出力する。単一自己完結ファイルの前提上、
# バンドル後に BASH_SOURCE 経由で外部の i18n/*.sh を実行時 source することはできないため。
write_common_lib_inlined() {
  local lib_path="$1"
  local in_i18n_block=false
  local line

  while IFS= read -r line || [[ -n "$line" ]]; do
    if [[ "$line" == *"=== i18n-load:begin ==="* ]]; then
      in_i18n_block=true
      echo 'BWQA_LANG_RESOLVED="$(bwqa_detect_lang)"' >> "$TMP_OUT"
      echo 'case "$BWQA_LANG_RESOLVED" in' >> "$TMP_OUT"
      local lang_file lang
      for lang_file in "$REPO_ROOT"/lib/i18n/*.sh; do
        lang="$(basename "$lang_file" .sh)"
        echo "  $lang)" >> "$TMP_OUT"
        cat "$lang_file" >> "$TMP_OUT"
        echo "  ;;" >> "$TMP_OUT"
      done
      echo '  *)' >> "$TMP_OUT"
      cat "$REPO_ROOT/lib/i18n/en.sh" >> "$TMP_OUT"
      echo '  ;;' >> "$TMP_OUT"
      echo 'esac' >> "$TMP_OUT"
      continue
    fi
    if [[ "$line" == *"=== i18n-load:end ==="* ]]; then
      in_i18n_block=false
      continue
    fi
    if $in_i18n_block; then
      continue
    fi
    echo "$line" >> "$TMP_OUT"
  done < "$lib_path"
}

# ヘッダーと本体を動的に判別して連結する
in_header=true
in_source_block=false
in_body=false
libs_written=false

while IFS= read -r line || [[ -n "$line" ]]; do
  # source 行や shellcheck source コメント行はスキップ
  if [[ "$line" =~ ^[[:space:]]*#[[:space:]]*shellcheck[[:space:]]+source= || "$line" =~ ^[[:space:]]*source[[:space:]]+ ]]; then
    if $in_header; then
      in_header=false
      in_source_block=true
    fi
    continue
  fi

  # source ブロックが終わった後の最初の非空行から本体とみなす
  if $in_source_block; then
    if [[ -z "${line// /}" ]]; then
      # source ブロック直後の空行はスキップ
      continue
    else
      in_source_block=false
      in_body=true
    fi
  fi

  if $in_header; then
    # BWQA_ROOT はバンドル版では未使用になり shellcheck 警告 (SC2034) を引き起こすため除外する
    if [[ "$line" =~ ^[[:space:]]*BWQA_ROOT= ]]; then
      continue
    fi

    # : "${BWQA_VERSION:=dev}" の行を処理
    if [[ "$line" == *'BWQA_VERSION:=dev'* ]]; then
      if [[ -n "${VERSION:-}" ]]; then
        echo "BWQA_VERSION=\"$VERSION\"" >> "$TMP_OUT"
      else
        echo "$line" >> "$TMP_OUT"
      fi
    else
      echo "$line" >> "$TMP_OUT"
    fi
  else
    # ヘッダー部分が終わったら、まず lib/ ファイルを一括して書き込む
    if ! $libs_written; then
      echo "" >> "$TMP_OUT"
      for lib in "${LIBS[@]}"; do
        lib_path="$REPO_ROOT/$lib"
        if [[ ! -f "$lib_path" ]]; then
          echo "エラー: ライブラリファイルが見つかりません: $lib_path" >&2
          exit 1
        fi
        echo "# === START $lib ===" >> "$TMP_OUT"
        if [[ "$lib" == "lib/common.sh" ]]; then
          write_common_lib_inlined "$lib_path"
        else
          cat "$lib_path" >> "$TMP_OUT"
        fi
        echo "# === END $lib ===" >> "$TMP_OUT"
        echo "" >> "$TMP_OUT"
      done
      libs_written=true
    fi
    
    if $in_body; then
      echo "$line" >> "$TMP_OUT"
    fi
  fi
done < "$REPO_ROOT/bin/bw-quickaccess"

# 成果物を所定のパスに移動して実行権限を付与
mv "$TMP_OUT" "$OUTPUT_PATH"
chmod +x "$OUTPUT_PATH"

echo "ビルド成功: $OUTPUT_PATH"

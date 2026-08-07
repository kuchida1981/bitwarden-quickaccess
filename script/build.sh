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

    # : "${BWQA_VERSION:=dev}" の行の直前に VERSION 環境変数が指定されていれば書き込む
    if [[ "$line" == *'BWQA_VERSION:=dev'* ]]; then
      if [[ -n "${VERSION:-}" ]]; then
        echo "BWQA_VERSION=\"$VERSION\"" >> "$TMP_OUT"
      fi
    fi
    echo "$line" >> "$TMP_OUT"
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
        cat "$lib_path" >> "$TMP_OUT"
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

# 成果物を所定 of パスに移動して実行権限を付与
mv "$TMP_OUT" "$OUTPUT_PATH"
chmod +x "$OUTPUT_PATH"

echo "ビルド成功: $OUTPUT_PATH"

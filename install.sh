#!/usr/bin/env bash
# install.sh
# Kuchida1981/bitwarden-quickaccess のリリースアセット(bw-quickaccess)をダウンロード・インストールするスクリプト

set -euo pipefail

# デフォルト値
PREFIX="$HOME/.local"
VERSION=""
OWNER="kuchida1981"
REPO="bitwarden-quickaccess"

print_usage() {
  cat <<EOF
Usage: install.sh [options]

Options:
  --prefix <path>   Installation prefix path (default: \$HOME/.local)
  --version <tag>   Specific version (release tag) to install (default: latest release)
  -h, --help        Show this help message and exit
EOF
}

# 引数の処理
while [[ $# -gt 0 ]]; do
  case "$1" in
    --)
      shift
      break
      ;;
    --prefix)
      if [[ -z "${2:-}" ]]; then
        echo "Error: --prefix option requires an argument." >&2
        exit 1
      fi
      PREFIX="$2"
      shift 2
      ;;
    --version)
      if [[ -z "${2:-}" ]]; then
        echo "Error: --version option requires an argument." >&2
        exit 1
      fi
      VERSION="$2"
      shift 2
      ;;
    -h|--help)
      print_usage
      exit 0
      ;;
    *)
      echo "Error: Unknown option $1" >&2
      print_usage >&2
      exit 1
      ;;
  esac
done

# ダウンロードURLの構築
if [[ -n "$VERSION" ]]; then
  DOWNLOAD_URL="https://github.com/${OWNER}/${REPO}/releases/download/${VERSION}/bw-quickaccess"
else
  DOWNLOAD_URL="https://github.com/${OWNER}/${REPO}/releases/latest/download/bw-quickaccess"
fi

TARGET_BIN="$PREFIX/bin/bw-quickaccess"
OLD_VERSION=""

# 既存のバージョンを確認
if [[ -x "$TARGET_BIN" ]]; then
  OLD_VER_RAW=$("$TARGET_BIN" --version 2>/dev/null || true)
  if [[ -n "$OLD_VER_RAW" ]]; then
    OLD_VERSION="${OLD_VER_RAW##* }"
  fi
  if [[ -z "$OLD_VERSION" ]]; then
    OLD_VERSION="unknown"
  fi
fi

# 一時ファイルを用意してダウンロード
TEMP_FILE=$(mktemp)
trap 'rm -f "$TEMP_FILE"' EXIT

echo "Downloading bw-quickaccess from $DOWNLOAD_URL ..."
if ! curl -fsSL "$DOWNLOAD_URL" -o "$TEMP_FILE"; then
  echo "Error: Failed to download bw-quickaccess from $DOWNLOAD_URL" >&2
  exit 1
fi

# ダウンロードファイルの検証
if [[ "$(head -c 2 "$TEMP_FILE")" != "#!" ]]; then
  echo "Error: Downloaded file does not appear to be a valid script (missing shebang)." >&2
  exit 1
fi

# インストール先ディレクトリの作成と配置
mkdir -p "$PREFIX/bin"
mv "$TEMP_FILE" "$TARGET_BIN"
chmod +x "$TARGET_BIN"

# インストール後の新バージョン確認
NEW_VER_RAW=$("$TARGET_BIN" --version 2>/dev/null || true)
NEW_VERSION=""
if [[ -n "$NEW_VER_RAW" ]]; then
  NEW_VERSION="${NEW_VER_RAW##* }"
fi
if [[ -z "$NEW_VERSION" ]]; then
  NEW_VERSION="unknown"
fi

if [[ -n "$OLD_VERSION" ]]; then
  echo "bw-quickaccess を $OLD_VERSION から $NEW_VERSION に更新しました。"
else
  echo "bw-quickaccess $NEW_VERSION をインストールしました。"
fi

# PATH の確認
PREFIX_BIN_ABS=""
if [[ -d "$PREFIX/bin" ]]; then
  PREFIX_BIN_ABS="$(cd "$PREFIX/bin" && pwd)"
fi

if [[ -z "$PREFIX_BIN_ABS" ]]; then
  PREFIX_BIN_ABS="$PREFIX/bin"
fi

case ":$PATH:" in
  *":$PREFIX_BIN_ABS:"*)
    # すでに PATH に含まれているので警告なし
    ;;
  *)
    # 警告メッセージを表示
    cat <<EOF >&2

[WARNING] $PREFIX_BIN_ABS が PATH に含まれていません。
コマンドをどこからでも実行できるようにするには、シェル設定ファイル(例: ~/.bashrc や ~/.zshrc)に以下の行を追加してください:

  export PATH="$PREFIX_BIN_ABS:\$PATH"

EOF
    ;;
esac

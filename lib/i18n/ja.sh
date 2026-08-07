# bw-quickaccess: メッセージ文字列(日本語)
# このファイルは lib/common.sh から自動的に source される。直接 source しない。

BWQA_MSG_ERR_PREFIX="エラー: %s"

# lib/preflight.sh
BWQA_MSG_PREFLIGHT_CMD_NOT_FOUND="必須コマンド '%s' が見つかりません。%s"
BWQA_MSG_PREFLIGHT_BW_INSTALL_HINT="https://bitwarden.com/help/cli/ を参照してインストールしてください(例: brew install bitwarden-cli)。"
BWQA_MSG_PREFLIGHT_JQ_INSTALL_HINT="'brew install jq' または各ディストリのパッケージマネージャでインストールしてください。"
BWQA_MSG_PREFLIGHT_FZF_INSTALL_HINT="'brew install fzf' または各ディストリのパッケージマネージャでインストールしてください。"
BWQA_MSG_PREFLIGHT_FZF_VERSION_UNKNOWN="fzf のバージョンを取得できませんでした。fzf %s 以上をインストールしてください(例: brew install fzf)。"
BWQA_MSG_PREFLIGHT_FZF_VERSION_TOO_OLD="fzf のバージョンが古すぎます(検出: %s / 必要: %s 以上)。'brew upgrade fzf' 等でアップグレードしてください。"
BWQA_MSG_PREFLIGHT_OS_UNSUPPORTED="サポート対象外の OS です(%s)。macOS または Linux(デスクトップ環境)のみサポートします。"
BWQA_MSG_PREFLIGHT_DISPLAY_NOT_FOUND="Wayland/X11 のディスプレイが検出できませんでした。デスクトップ GUI 環境で実行してください(ヘッドレス/SSH専用環境は非対応です)。"
BWQA_MSG_PREFLIGHT_MACOS_BUILTIN_HINT="macOS には標準搭載されているはずです。PATH を確認してください。"
BWQA_MSG_PREFLIGHT_WL_COPY_NOT_FOUND="wl-copy が見つかりません。'apt install wl-clipboard' 等でインストールしてください。"
BWQA_MSG_PREFLIGHT_XCLIP_NOT_FOUND="xclip または xsel が見つかりません。'apt install xclip' 等でインストールしてください。"
BWQA_MSG_PREFLIGHT_SECRET_TOOL_NOT_FOUND="secret-tool が見つかりません。'apt install libsecret-tools' 等でインストールしてください。"
BWQA_MSG_PREFLIGHT_KEYRING_SELFTEST_FAILED="警告: keyring バックエンド(GNOME Keyring/KWallet 等)への疎通に失敗しました。session のキャッシュは無効化し、毎回マスターパスワードの入力を求めます。"

# lib/session.sh
BWQA_MSG_SESSION_UNLOCKING="vaultのロックを解除しています..."
BWQA_MSG_SESSION_UNLOCK_FAILED="bw unlock に失敗しました。マスターパスワードを確認してください。"
BWQA_MSG_SESSION_EMPTY="bw unlock が空の session を返しました。"
BWQA_MSG_SESSION_REAUTH="session が無効になっているため、再認証します。"
BWQA_MSG_SESSION_BW_CMD_FAILED="bw %s に失敗しました: %s"
BWQA_MSG_SESSION_CACHE_CLEARED="session のキャッシュを破棄しました。"

# lib/search.sh
BWQA_MSG_SEARCH_LOADING_ITEMS="vaultのアイテム一覧を読み込んでいます..."
BWQA_MSG_SEARCH_FETCH_FAILED="vault アイテムの取得に失敗しました。"
BWQA_MSG_SEARCH_FZF_HEADER="Enter: アイテムを選択  ctrl-o: ユーザー名  ctrl-r: パスワード  ctrl-t: TOTP を直接コピー  Esc: 終了"

# lib/fields.sh
BWQA_MSG_FIELDS_LOADING_ITEM="アイテム情報を取得しています..."
BWQA_MSG_FIELDS_ITEM_FETCH_FAILED="アイテム情報の取得に失敗しました。"
BWQA_MSG_FIELDS_NO_COPYABLE_FIELDS="コピー可能なフィールドがありません: %s"
BWQA_MSG_FIELDS_ROW_USERNAME="ユーザー名をコピー (ctrl-o)"
BWQA_MSG_FIELDS_ROW_PASSWORD="パスワードをコピー (ctrl-r)"
BWQA_MSG_FIELDS_ROW_TOTP="TOTP をコピー (ctrl-t)"
BWQA_MSG_FIELDS_FZF_HEADER="Enter: 選択中の項目をコピー  ctrl-r: password  ctrl-o: username  ctrl-t: totp  Esc: 検索へ戻る  q: 終了"
BWQA_MSG_FIELDS_LABEL_USERNAME="ユーザー名"
BWQA_MSG_FIELDS_LABEL_PASSWORD="パスワード"
BWQA_MSG_FIELDS_LABEL_TOTP="TOTP"
BWQA_MSG_FIELDS_COPYING="コピー中..."
BWQA_MSG_FIELDS_COPY_FAILED="コピーに失敗しました"
BWQA_MSG_FIELDS_VALUE_NOT_SET="%sは設定されていません"
BWQA_MSG_FIELDS_COPY_SUCCESS="%sをコピーしました"

# bin/bw-quickaccess
BWQA_MSG_USAGE_TEXT="使い方: bw-quickaccess [lock]

  (引数なし)   vault アイテムを検索し、フィールドをクリップボードへコピーする
  lock         キャッシュされた session を破棄する(次回起動時に再度マスターパスワードを要求する)
  -v, --version   バージョンを表示する
  -h, --help   このヘルプを表示する"
BWQA_MSG_UNKNOWN_ARG="不明な引数です: %s"

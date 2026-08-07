## Context

`lib/*.sh` は `bin/bw-quickaccess` から固定順(`common.sh` → `preflight.sh` → `clipboard.sh` → `session.sh` → `search.sh` → `fields.sh`)で `source` される。メッセージ出力は `lib/common.sh` の `bwqa_log()`/`bwqa_die()` に集約されており、各 lib ファイルはこの2関数を経由して日本語固定文字列を渡している(現状12箇所)。加えて `lib/search.sh` と `lib/fields.sh` の fzf 起動オプション(`--prompt`/`--header`)にも日本語文字列がハードコードされている。

制約: macOS 標準 `/bin/bash` は 3.2 系で `declare -A`(連想配列)非対応。このプロジェクトは README で全ての外部コマンド依存を明示・preflight チェックする方針を取っており、新規の必須外部コマンド(gettext 等)を増やしたくない。配布は無ビルド(clone してそのまま実行 / curl\|bash インストール)が前提。

## Goals / Non-Goals

**Goals:**
- `bwqa_log`/`bwqa_die` 経由の全メッセージと fzf の `--prompt`/`--header` 文言を日本語・英語で切り替え可能にする
- 新規外部依存・ビルドステップを追加しない(bash 3.2 でも動作する)
- `BWQA_LANG` 環境変数による明示的な言語強制、および `LANG`/`LC_ALL` からの自動判定をサポートする
- 対応言語ファイルが無い場合は英語にフォールバックする

**Non-Goals:**
- 日本語・英語以外の言語追加(仕組みとしては拡張可能だが、今回のスコープでは作成しない)
- 複数形(plural forms)処理の一般化(該当メッセージが現状ないため)
- 設定ファイルによる永続的な言語設定(環境変数のみとする)

## Decisions

### 1. メッセージ管理: `lib/i18n/<lang>.sh` + `BWQA_MSG_*` 変数(プレフィックス方式)

`declare -A` を使わず、`BWQA_MSG_ERR_BW_NOT_FOUND="..."` のようにフラットな変数として定義する。理由:
- bash 3.2 で動作する(連想配列は bash 4+ 限定)
- 既存の `lib/*.sh` を `source` するパターンと完全に一致し、レビューしやすい
- 変数名は意味のある識別子にする(例: `BWQA_MSG_ERR_CMD_NOT_FOUND`, `BWQA_MSG_LOADING_ITEMS`, `BWQA_MSG_FZF_HEADER_SEARCH`)

**代替案: gettext/ngettext** — 却下(理由は proposal.md 参照。macOS 非標準の追加依存とビルドステップが既存の配布モデルと合わない)。

**代替案: 連想配列** — 却下(bash 3.2 非対応)。

### 2. 言語判定順序と実装場所

`lib/common.sh` に `bwqa_detect_lang()` を追加し、`bin/bw-quickaccess` からの `source lib/common.sh` 直後、他の lib ファイルを source する前に言語ファイルを読み込む。

判定順序:
1. `BWQA_LANG` 環境変数(`ja`/`en` のみ許容。他の値は無視して次へ)
2. `LC_ALL` → `LANG` の先頭2文字(`ja*` → `ja`、それ以外 → `en`)
3. 上記で判定できない、または対応する `lib/i18n/<lang>.sh` が存在しない場合は `en` にフォールバック

```
bin/bw-quickaccess
  source lib/common.sh          # bwqa_log/bwqa_die/bwqa_detect_lang を定義
  bwqa_detect_lang → BWQA_LANG_RESOLVED="ja" | "en"
  source lib/i18n/${BWQA_LANG_RESOLVED}.sh   # BWQA_MSG_* を定義
  source lib/preflight.sh ...   # 以降、BWQA_MSG_* を参照できる
```

`lib/i18n/*.sh` の source は `common.sh` の直後・他 lib より前に行う必要がある(`preflight.sh` 等が起動直後から `BWQA_MSG_*` を参照するため)。

### 3. 既存メッセージの移行

`bwqa_log "..."` / `bwqa_die "..."` の呼び出し箇所(12箇所)を、対応する `BWQA_MSG_*` 変数参照に置き換える。動的な埋め込み値(コマンド名・バージョン番号など)がある文字列は `printf` 形式のテンプレート変数にする(例: `BWQA_MSG_ERR_CMD_NOT_FOUND='必須コマンド '\''%s'\'' が見つかりません。%s'` を `printf` で展開)。

fzf の `--prompt`/`--header` も同様に `BWQA_MSG_FZF_*` 変数に切り出す。`--prompt='vault> '` のような非日本語文字列(記号のみ)は変更不要。

### 4. README

`README.md` を英語版に書き換え、既存の日本語版内容を `README.ja.md` として新規作成する。両ファイルの冒頭に相互リンク(`Read this in Japanese / 日本語版はこちら`)を設置する。

## Risks / Trade-offs

- [Risk] `BWQA_MSG_*` 変数の命名が lib ファイル間で衝突・重複する → [Mitigation] 呼び出し元のファイル名や機能をプレフィックスに含めて一意にする(例: `BWQA_MSG_PREFLIGHT_*`, `BWQA_MSG_SESSION_*`)。命名規則を tasks.md 内の実装時に一覧化する
- [Risk] 新しい `bwqa_log`/`bwqa_die` 呼び出しが将来追加された際、i18n 変数化を忘れてハードコード日本語が復活する → [Mitigation] CI に「`bwqa_log\|bwqa_die` へのリテラル日本語文字列直渡し」を grep で検出するチェックを追加することを検討(このchangeのスコープ外、フォローアップ課題として明記)
- [Risk] `LANG` が未設定の CI 環境(GitHub Actions 等)でテストの言語判定が不安定になる → [Mitigation] bats テストでは `BWQA_LANG=en` 等を明示的に設定してテストする
- [Trade-off] gettext の持つ「翻訳者向けツール(.po エディタ連携、fuzzy match)」は得られない。2言語・数十文字列規模では十分にシンプルな方式で足りると判断

## Open Questions

- fzf のヘッダー文言は変更頻度が低いため、今回のスコープで全て i18n 化する前提だが、将来的にキーバインド変更(alt-u/alt-p → ctrl-o/ctrl-r、直近の fix-copy-keybind-meta-dependency 変更)と同時にメンテナンスが必要になる。今回は現状のキーバインド文言をそのまま2言語化する

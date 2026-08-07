## 1. 言語ファイルの雛形と言語判定ロジック

- [x] 1.1 `lib/i18n/ja.sh` と `lib/i18n/en.sh` を新規作成し、既存メッセージ(`bwqa_log`/`bwqa_die` 呼び出し12箇所 + fzf `--prompt`/`--header`)から `BWQA_MSG_*` 変数名の一覧を洗い出して定義する(値はまず日本語版のみ埋め、英語版は 2. で翻訳)
- [x] 1.2 `lib/common.sh` に `bwqa_detect_lang()` を追加する(`BWQA_LANG` → `LC_ALL` → `LANG` の順で判定し、`ja` で始まらなければ `en`。対応する `lib/i18n/<lang>.sh` が存在しない場合は `en` にフォールバック)
- [x] 1.3 実装変更: `bin/bw-quickaccess` 側での明示的な source 呼び出しではなく、`lib/common.sh` 自身が末尾で `bwqa_detect_lang` を呼び `lib/i18n/${BWQA_LANG_RESOLVED}.sh` を自動 source するように変更した(bats テストが `lib/common.sh` を直接 source するため、production/test 双方で自動的に i18n が有効になるようにするための設計変更。design.md に反映済み)

## 2. 既存メッセージの i18n 化

- [x] 2.1 `lib/i18n/en.sh` に全 `BWQA_MSG_*` の英語訳を追加する
- [x] 2.2 `lib/common.sh`(`bwqa_die` 内のエラープレフィックス)を `BWQA_MSG_*` 参照に置き換える
- [x] 2.3 `lib/preflight.sh` の `bwqa_log`/`bwqa_die` 呼び出し(必須コマンド未検出・fzf バージョン不足・OS非対応・ディスプレイ未検出・clipboard/keychain ツール未検出・keyring 疎通失敗の警告)を `BWQA_MSG_*` 参照に置き換える(agy)
- [x] 2.4 `lib/session.sh` の `bwqa_log`/`bwqa_die` 呼び出し(unlock中・unlock失敗・session空・再認証・bw失敗・キャッシュ破棄)を `BWQA_MSG_*` 参照に置き換える(agy)
- [x] 2.5 `lib/search.sh` の `bwqa_log`/`bwqa_die` 呼び出し(アイテム読み込み中・取得失敗)および fzf `--header` の文言を `BWQA_MSG_*` 参照に置き換える(`--prompt='vault> '` は記号のみのため対象外)(agy)
- [x] 2.6 `lib/fields.sh` の `bwqa_log`/`bwqa_die` 呼び出し・fzf `--header`・jq 行ラベル(`jq --arg` 経由)・`bwqa_field_label`・コピー結果ステータス文言を `BWQA_MSG_*` 参照に置き換える(範囲を拡張: 当初想定していなかった行ラベル/ステータスファイル文言も対象に含めた)(agy)
- [x] 2.7 `bin/bw-quickaccess` の `bwqa_print_usage`(ヘルプ文言)と、範囲を広げて「不明な引数です」メッセージも `BWQA_MSG_*` 参照に置き換える
- [x] 2.8 動的な埋め込み値(コマンド名・バージョン番号等)を含むメッセージは `printf` テンプレート形式で `BWQA_MSG_*` を定義し、呼び出し側で `printf` 展開するように統一する(`# shellcheck disable=SC2059` を付与)

## 3. テスト

- [x] 3.1 `test/lib/common.bats` に `bwqa_detect_lang` の判定ロジック(`BWQA_LANG` 優先・`LC_ALL`/`LANG` 判定・フォールバック)のテストケースを追加する(agy)
- [x] 3.2 `test/helpers/stub.bash` の `bwqa_test_stub_setup()` で `BWQA_LANG="${BWQA_LANG:-ja}"` をデフォルト設定し、CI のロケールに依存せず既存テスト(日本語文字列の assertion)が安定して通るようにした(既存 assertion の書き換えは不要だった)

## 4. ドキュメント

- [x] 4.1 `README.md` を英語版に書き換える(Claude Code が翻訳品質確保のため直接実施。新機能である `BWQA_LANG` 表示言語切り替えの説明も追記)
- [x] 4.2 既存の日本語 README の内容を `README.ja.md` として新規作成する(同様に `BWQA_LANG` の説明を追記)
- [x] 4.3 `README.md` と `README.ja.md` の冒頭に相互リンク("Read this in English / [日本語版はこちら]" 形式)を追加する
- [x] 4.4 `openspec/specs/` に新規 capability `message-localization` の spec が反映されるよう、change アーカイブ時の delta sync を確認する(archive フェーズで実施)

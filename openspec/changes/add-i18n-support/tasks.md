## 1. 言語ファイルの雛形と言語判定ロジック

- [ ] 1.1 `lib/i18n/ja.sh` と `lib/i18n/en.sh` を新規作成し、既存メッセージ(`bwqa_log`/`bwqa_die` 呼び出し12箇所 + fzf `--prompt`/`--header`)から `BWQA_MSG_*` 変数名の一覧を洗い出して定義する(値はまず日本語版のみ埋め、英語版は 2. で翻訳)
- [ ] 1.2 `lib/common.sh` に `bwqa_detect_lang()` を追加する(`BWQA_LANG` → `LC_ALL` → `LANG` の順で判定し、`ja` で始まらなければ `en`。対応する `lib/i18n/<lang>.sh` が存在しない場合は `en` にフォールバック)
- [ ] 1.3 `bin/bw-quickaccess` で `lib/common.sh` の source 直後に `bwqa_detect_lang` を呼び出し、`lib/i18n/${BWQA_LANG_RESOLVED}.sh` を source する処理を追加する(他の lib ファイルより前に実行されること)

## 2. 既存メッセージの i18n 化

- [ ] 2.1 `lib/i18n/en.sh` に全 `BWQA_MSG_*` の英語訳を追加する
- [ ] 2.2 `lib/common.sh`(`bwqa_die` 内のエラープレフィックス)を `BWQA_MSG_*` 参照に置き換える
- [ ] 2.3 `lib/preflight.sh` の `bwqa_log`/`bwqa_die` 呼び出し(必須コマンド未検出・fzf バージョン不足・OS非対応・ディスプレイ未検出・clipboard/keychain ツール未検出・keyring 疎通失敗の警告)を `BWQA_MSG_*` 参照に置き換える
- [ ] 2.4 `lib/session.sh` の `bwqa_log`/`bwqa_die` 呼び出し(unlock中・unlock失敗・session空・再認証・bw失敗・キャッシュ破棄)を `BWQA_MSG_*` 参照に置き換える
- [ ] 2.5 `lib/search.sh` の `bwqa_log`/`bwqa_die` 呼び出し(アイテム読み込み中・取得失敗)および fzf `--prompt`/`--header` の文言を `BWQA_MSG_*` 参照に置き換える
- [ ] 2.6 `lib/fields.sh` の `bwqa_log`/`bwqa_die` 呼び出し(アイテム情報取得中・取得失敗・コピー可能フィールドなし)および fzf `--prompt`/`--header` の文言を `BWQA_MSG_*` 参照に置き換える
- [ ] 2.7 `bin/bw-quickaccess` の `bwqa_print_usage`(ヘルプ文言)を `BWQA_MSG_*` 参照に置き換える
- [ ] 2.8 動的な埋め込み値(コマンド名・バージョン番号等)を含むメッセージは `printf` テンプレート形式で `BWQA_MSG_*` を定義し、呼び出し側で `printf` 展開するように統一する

## 3. テスト

- [ ] 3.1 `test/lib/common.bats` に `bwqa_detect_lang` の判定ロジック(`BWQA_LANG` 優先・`LANG`/`LC_ALL` 判定・フォールバック)のテストケースを追加する
- [ ] 3.2 既存の `test/lib/*.bats` で `BWQA_LANG=en`(または `ja`)を明示的に設定し、メッセージ言語に依存せずテストが安定して通ることを確認する(ハードコードされた日本語文字列の assertion があれば `BWQA_MSG_*` 参照に置き換える)

## 4. ドキュメント

- [ ] 4.1 `README.md` を英語版に書き換える
- [ ] 4.2 既存の日本語 README の内容を `README.ja.md` として新規作成する
- [ ] 4.3 `README.md` と `README.ja.md` の冒頭に相互リンク("Read this in Japanese / 日本語版はこちら")を追加する
- [ ] 4.4 `openspec/specs/` に新規 capability `message-localization` の spec が反映されるよう、change アーカイブ時の delta sync を確認する(archive フェーズで実施)

## 1. 共通基盤

- [x] 1.1 `lib/common.sh` に `BWQA_COPY_STATUS_FILE` 定数を追加する(`BWQA_ERROR_LOG_FILE` と同じパターンで `$BWQA_CACHE_DIR` 配下に配置)
- [x] 1.2 `lib/preflight.sh` の `bwqa_check_fzf_version()` の `required` を `"0.35.0"` から `"0.37.0"` に変更する

## 2. bwqa_copy_field_internal の結果メッセージ実装

- [x] 2.1 `bw get <field> <item_id>` 呼び出しの終了コードを `|| true` で握りつぶさず変数に保持するよう変更する
- [x] 2.2 「成功」「フィールド未設定(終了コード0かつ値が空)」「`bw` コマンド失敗(終了コード非ゼロ)」の3パターンを区別する分岐を実装する
- [x] 2.3 各パターンに応じた固定文言のメッセージ(アイテム名・フィールド値等の機密情報を含まない)を `BWQA_COPY_STATUS_FILE` に上書きで書き込む
- [x] 2.4 フィールド種別(username/password/totp)ごとの日本語ラベル(ユーザー名/パスワード/TOTP)を `bwqa_build_field_rows()` の表記に合わせてメッセージに使う

## 3. フィールド選択画面のフィードバック表示

- [x] 3.1 `bwqa_run_field_screen()` の fzf 呼び出しに `--border=rounded` と `--border-label` を追加する(初期値は空文字または固定の案内文言とし、過去の実行で残った状態ファイルの中身は起動時に読みに行かない)
- [x] 3.2 `enter` / `ctrl-p` / `ctrl-u` / `ctrl-t` の各 `--bind` に `+transform-border-label(cat "$BWQA_COPY_STATUS_FILE" 2>/dev/null)` を連結する
- [x] 3.3 `--height=80% --reverse` レイアウトでの見た目を実機(tmux)で確認し、必要なら調整する

## 4. ドキュメント更新

- [x] 4.1 `README.md` の必要な `fzf` バージョン記載を `0.37.0` 以上に更新する

## 5. テスト

- [x] 5.1 `test/lib/fields.bats` に `bwqa_copy_field_internal` の3パターン(成功/フィールド未設定/`bw` コマンド失敗)それぞれで `BWQA_COPY_STATUS_FILE` に期待通りのメッセージが書き込まれることを検証するテストを追加する
- [x] 5.2 既存の bats テストスイートを実行し、全て通過することを確認する

## 6. 動作確認

- [x] 6.1 tmux 上で実際に fzf を起動し、ctrl-u/ctrl-p/ctrl-t それぞれで成功時・フィールド未設定時のフィードバック表示を目視確認する
- [x] 6.2 `bw` コマンド失敗ケース(スタブで再現)でのフィードバック表示を目視確認する

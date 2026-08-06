## 1. lib/fields.sh のコメント更新

- [ ] 1.1 冒頭コメントの「アイテム ID / session token はコマンド文字列に埋め込まない」という記述を、item id は非秘匿情報として `{1}` 埋め込みを許容し、session token のみコマンド文字列に埋め込まない旨に修正する(design.md の Decisions 参照)

## 2. lib/search.sh の実装

- [ ] 2.1 `bwqa_run_search_screen()` の fzf 呼び出しに `--border=rounded --border-label=''` を追加する
- [ ] 2.2 `bwqa_run_search_screen()` の fzf 呼び出し全体を subshell で包み、起動前に `export BW_SESSION="$BWQA_SESSION"` する(`lib/fields.sh` の `bwqa_run_field_screen()` と同じパターン)
- [ ] 2.3 `ctrl-u`/`ctrl-p`/`ctrl-t` の `execute-silent` バインドを追加する。item id は `{1}` でコマンド文字列に埋め込んで `"$BWQA_SELF" __copy-field <field>` を呼び出し、`+transform-border-label(cat "$BWQA_COPY_STATUS_FILE")` を連結する(`lib/fields.sh` の対応するバインドを参考にする)
- [ ] 2.4 `--header` を更新し、`ctrl-u`/`ctrl-p`/`ctrl-t` による直接コピーの説明を追加する(既存の `Enter: アイテムを選択  Esc: 終了` は維持する)

## 3. 動作確認

- [ ] 3.1 tmux 等の実機で、検索画面から `ctrl-u`/`ctrl-p`/`ctrl-t` を押して直接コピーできること、画面が閉じずに続けて別アイテムを検索・コピーできることを目視確認する
- [ ] 3.2 対象フィールドが存在しないアイテムでキーバインドを押した場合に、border-label へフィールド未設定のメッセージが表示されることを確認する
- [ ] 3.3 `--border` 追加による検索画面のレイアウト崩れ(表示行数の減少等)がないか確認し、必要であれば `--height` を調整する
- [ ] 3.4 Enter によるフィールド選択画面への遷移が従来どおり機能することを確認する(直接コピーの追加が既存フローを壊していないことの確認)

## 4. ドキュメント更新

- [ ] 4.1 README.md の操作方法セクションに、検索画面からの直接コピー(`ctrl-u`/`ctrl-p`/`ctrl-t`)を追記する

## 5. テスト

- [ ] 5.1 既存の `test/lib/search.bats` が変更後も通ることを確認する(fzf 対話画面自体はスコープ外の方針を維持し、新規の純粋ロジックが追加された場合のみテストを追加する)

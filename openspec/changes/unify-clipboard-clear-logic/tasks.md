## 1. ClipboardGuard の拡張

- [ ] 1.1 `app/src-tauri/src/backend/clipboard_guard.rs` の `ClipboardGuard` に `pub fn last_value(&self) -> Option<String>`(現在保持している値のクローンを返す)を追加し、`cargo build -p bw-quickaccess-gui` が通ることを確認する
- [ ] 1.2 `last_value()` のユニットテスト(未setの場合は`None`、`set`後は`Some(value)`、`clear`後は`None`を返すこと)を `clipboard_guard.rs` の `#[cfg(test)] mod tests` に追加し、`cargo test -p bw-quickaccess-gui clipboard_guard` が通ることを確認する

## 2. clear_clipboard_if_owned の統一

- [ ] 2.1 `app/src-tauri/src/commands.rs` の `clear_clipboard_if_owned` のシグネチャを `pub fn clear_clipboard_if_owned(app: &tauri::AppHandle, guard: &ClipboardGuard, expected: &str)` に変更し、内部ロジックを「`current == expected` の場合にクリアして `guard.clear_if_matches(expected)` を呼ぶ」形に書き換える(`guard.should_clear()` の呼び出しは削除する)
- [ ] 2.2 `lock()` コマンド内の呼び出し箇所を `if let Some(expected) = guard.last_value() { clear_clipboard_if_owned(&app, &guard, &expected); }` に変更する
- [ ] 2.3 `copy_field` 内の遅延クリアのインライン実装(`tokio::time::sleep` 後の読み取り・比較・書き込みブロック)を削除し、`clear_clipboard_if_owned(&app_for_clear, &guard_for_clear, &expected_for_clear)` の呼び出しに置き換える
- [ ] 2.4 `cargo build -p bw-quickaccess-gui` と `cargo clippy --all-targets -- -D warnings` がいずれもエラー・警告なく通ることを確認する

## 3. 回帰テストの追加

- [ ] 3.1 `clipboard_guard.rs` に、`guard.set(V1)` の後に `guard.set(V2)` した状態で `clear_if_matches(V1)` を呼んでも `last_value()` が `Some(V2)` のまま変わらないことを検証するユニットテストを追加する(design.md の「30秒以内に2回コピー」シナリオにおける、先発のexpectedで後発の値を誤ってクリアしないことの担保)
- [ ] 3.2 `commands.rs` の `#[cfg(test)] mod tests` に、`clear_clipboard_if_owned` を直接呼び出し、`expected` が `guard` の現在値と異なる場合(＝後発の値で上書きされた場合)にクリップボードもguard状態もクリアされない(guardが`clear_if_matches`により意図せず変更されない)ことを検証するテストを追加する(実クリップボードI/Oが必要な場合はモック不可のため、`ClipboardGuard` 単体のロジック検証で代替してよい)
- [ ] 3.3 `cargo test -p bw-quickaccess-gui` を実行し、既存テストを含め全て成功することを確認する

## 4. ドキュメント確認

- [ ] 4.1 `openspec/specs/credential-copy-actions/spec.md` の既存要求(30秒後クリア・上書き時スキップ)が今回の実装変更後も引き続き成立していることを確認する(要求文言自体の変更は不要)
- [ ] 4.2 README.md・CLAUDE.md に今回の変更で更新が必要な記述がないか確認する(通常は変更不要のはずだが、コーディング規約への影響がないか一応チェックする)

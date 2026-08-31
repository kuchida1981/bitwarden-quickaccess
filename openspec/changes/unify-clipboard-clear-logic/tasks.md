## 1. ClipboardGuard の拡張

- [x] 1.1 `app/src-tauri/src/backend/clipboard_guard.rs` の `ClipboardGuard` に `pub fn last_value(&self) -> Option<String>`(現在保持している値のクローンを返す)を追加し、`cargo build -p bw-quickaccess-gui` が通ることを確認する
- [x] 1.2 `last_value()` のユニットテスト(未setの場合は`None`、`set`後は`Some(value)`、`clear`後は`None`を返すこと)を `clipboard_guard.rs` の `#[cfg(test)] mod tests` に追加し、`cargo test -p bw-quickaccess-gui clipboard_guard` が通ることを確認する

## 2. clear_clipboard_if_owned の統一

- [x] 2.1 `app/src-tauri/src/commands.rs` の `clear_clipboard_if_owned` のシグネチャを `pub fn clear_clipboard_if_owned(app: &tauri::AppHandle, guard: &ClipboardGuard, expected: &str)` に変更し、内部ロジックを「`current == expected` の場合にクリアして `guard.clear_if_matches(expected)` を呼ぶ」形に書き換える(`guard.should_clear()` の呼び出しは削除する)
- [x] 2.2 `lock()` コマンド内の呼び出し箇所を `if let Some(expected) = guard.last_value() { clear_clipboard_if_owned(&app, &guard, &expected); }` に変更する
- [x] 2.3 `copy_field` 内の遅延クリアのインライン実装(`tokio::time::sleep` 後の読み取り・比較・書き込みブロック)を削除し、`clear_clipboard_if_owned(&app_for_clear, &guard_for_clear, &expected_for_clear)` の呼び出しに置き換える
- [x] 2.3a (実装時に判明した追加対応) `app/src-tauri/src/main.rs` の `watch_idle_timeout`(アイドル自動ロック)内の呼び出し箇所も同じシグネチャ変更に追随させ、`lock()` と同じ `if let Some(expected) = guard.last_value() { ... }` パターンに揃える
- [x] 2.4 `cargo build -p bw-quickaccess-gui` と `cargo clippy --all-targets -- -D warnings` がいずれもエラー・警告なく通ることを確認する

## 2.5 未使用コードの削除(実装時に判明した追加対応)

- [x] 2.5.1 `clear_clipboard_if_owned` の統一により `ClipboardGuard::should_clear` が本番コード(`commands.rs`, `main.rs`)から一切呼ばれなくなっていることを確認する(`grep -rn "should_clear" app/src-tauri/src` で本体側の呼び出しがテスト以外に無いことを確認)
- [x] 2.5.2 `should_clear` メソッド本体を `clipboard_guard.rs` から削除し、`should_clear` に依存していた既存テスト(`should_clear_is_*`, `set_overwrites_previous_value`, `clear_if_matches_*`)を `last_value()` を使ったアサーションに書き換える(検証内容自体は変えず、アサーション手段だけ `last_value()` ベースに置き換える)
- [x] 2.5.3 `cargo build -p bw-quickaccess-gui` と `cargo clippy --all-targets -- -D warnings` が通ることを確認する

## 3. 回帰テストの追加

- [x] 3.1 `clipboard_guard.rs` に、`guard.set(V1)` の後に `guard.set(V2)` した状態で `clear_if_matches(V1)` を呼んでも `last_value()` が `Some(V2)` のまま変わらないことを検証するユニットテストを追加する(design.md の「30秒以内に2回コピー」シナリオにおける、先発のexpectedで後発の値を誤ってクリアしないことの担保)→ `clear_if_matches_does_not_clear_when_overwritten_by_newer_value` として実装済み
- [x] 3.2 (スコープ調整) `clear_clipboard_if_owned` は `tauri::AppHandle` を引数に取り、現状の環境ではモックできないため、`commands.rs` 側への直接テスト追加は行わない。回帰防止の核心ロジックは3.1の `ClipboardGuard` レベルのテストで担保済みと判断した(`clear_clipboard_if_owned` 自体は「文字列比較 + clear_if_matches呼び出し」のみの薄いラッパーであり、追加の分岐ロジックを持たないため)
- [x] 3.3 `cargo test -p bw-quickaccess-gui` を実行し、既存テストを含め全て成功することを確認する(65 passed; 0 failed)

## 4. ドキュメント確認

- [x] 4.1 `openspec/specs/credential-copy-actions/spec.md` の既存要求(30秒後クリア・上書き時スキップ)が今回の実装変更後も引き続き成立していることを確認する(要求文言自体の変更は不要)→ 確認済み、spec.mdの要求(L48-57)は今回の実装と完全に整合しており変更不要
- [x] 4.2 README.md・CLAUDE.md に今回の変更で更新が必要な記述がないか確認する(通常は変更不要のはずだが、コーディング規約への影響がないか一応チェックする)→ README.mdのクリップボード自動クリア(30秒)の記述は観測可能な挙動として変わらないため更新不要。CLAUDE.mdの該当記述も本変更のコーディング規約に影響しないため更新不要

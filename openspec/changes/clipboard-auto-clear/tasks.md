## 1. ClipboardGuard基盤の追加

- [x] 1.1 `app/src-tauri/src/backend/clipboard_guard.rs` を新設し、`ClipboardGuard` 構造体(`last_written: Arc<Mutex<Option<String>>>`)と `set`/`clear`/`should_clear` を実装する(`backend/idle.rs` の `IdleTimer` と同様のパターンを踏襲する)
- [x] 1.2 `app/src-tauri/src/backend/mod.rs` に新モジュールを追加する
- [x] 1.3 `app/src-tauri/src/main.rs` で `ClipboardGuard` を生成し `tauri::Builder::manage()` に登録する

## 2. コピー後の遅延自動クリア

- [x] 2.1 クリア遅延時間の定数(`CLIPBOARD_CLEAR_DELAY`, 30秒)を定義する(`IdleTimer::DEFAULT_IDLE_TIMEOUT` と同様の置き場所・書き方に揃える)
- [x] 2.2 `commands::copy_field` で `app.clipboard().write_text(value.clone())` 成功後に `guard.set(value.clone())` を呼ぶ
- [x] 2.3 `copy_field` の末尾で `tauri::async_runtime::spawn` により、`CLIPBOARD_CLEAR_DELAY` 経過後にクリアを試みる遅延タスクを起動する
- [x] 2.4 遅延タスク内で `app.clipboard().read_text()` の結果を `guard.should_clear(&current)` で判定し、true の場合のみ空文字列を書き込んで `guard.clear()` する(読み取り失敗時は何もしない)

## 3. ロック時の即時クリア

- [x] 3.1 「クリップボードの中身が期待値のままなら即座にクリアする」共通処理(例: `clear_clipboard_if_owned(app, guard)` 関数)を実装し、遅延タスクとロック側から再利用できるようにする
- [x] 3.2 `commands::lock` でロック成功後に即時クリア処理を呼ぶ
- [x] 3.3 `main.rs` の `watch_idle_timeout` でロック成功(`client.lock().await.is_ok()`)後に即時クリア処理を呼ぶ

## 4. テスト・検証

- [x] 4.1 `ClipboardGuard::should_clear` のユニットテストを追加する(未設定時/一致時/不一致時/`clear()`後の各ケース)
- [x] 4.2 `cd app/src-tauri && cargo build && cargo test && cargo clippy --all-targets -- -D warnings` をローカルで実行し、CI(`ci.yml`)と同じチェックが通ることを確認する
- [ ] 4.3 実機での手動確認: (a) コピー後30秒待つとクリアされる、(b) コピー後30秒以内に別の値をコピーしても上書き・消去されない、(c) コピー直後に `⌘L` でロックすると即座にクリアされる、(d) コピー直後にアイドルタイムアウトで自動ロックされると即座にクリアされる

## 5. ドキュメント更新

- [x] 5.1 `README.md` のクリップボードコピーに関する記述に、30秒後の自動クリアについて簡潔に追記する(`README.ja.md` にも同様に追記)

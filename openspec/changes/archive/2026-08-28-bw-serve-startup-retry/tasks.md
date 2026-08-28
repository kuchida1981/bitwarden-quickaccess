## 1. process.rs: 2フェーズ監視の追加

- [x] 1.1 `process.rs` に `MAX_STARTUP_ATTEMPTS`(=3)定数を追加する
- [x] 1.2 `StartupHandles`(`process_handle`, `monitor`, `exited: oneshot::Receiver<()>`, `confirm: oneshot::Sender<()>`)を新設する
- [x] 1.3 `spawn_supervised_for_startup(port, state)` を新設する。監視タスクを「`confirm` 受信前は exit を `exited` チャンネルへ通知するだけで `state` には触れない」「`confirm` 受信後は現行の `spawn_supervised_with_command` と同じ『予期せぬ終了→`state.set_error()`』監視に切り替える」の2フェーズ構造にする
- [x] 1.4 既存の `spawn_supervised` / `spawn_supervised_with_command` は変更せず温存し、`build_bw_serve_command` 等の共通パーツを `spawn_supervised_for_startup` からも再利用する

## 2. main.rs: リトライループの実装

- [x] 2.1 `start_backend` 内の「`pick_free_port()` → `spawn_supervised()` → `state.set_port()` → `sync_initial_status()`」を、`MAX_STARTUP_ATTEMPTS` 回までのリトライループに置き換える
- [x] 2.2 各試行で `tokio::select!` により `sync_initial_status(&client, &state)` の完了と `handles.exited` を競合させる。`exited` が先に発火した場合は `eprintln!` でログを出しつつ次の試行(新しいポート)へ進む
- [x] 2.3 `sync_initial_status` が先に完了した場合は `handles.confirm` を送信し、`ManagedProcess` への登録と `state.set_port(port)` を行って成功終了する
- [x] 2.4 全試行が早期終了で尽きた場合、`state.set_error(...)` でエラー状態を記録して終了する

## 3. テスト

- [x] 3.1 `spawn_supervised_for_startup` の単体テスト: `confirm` 送信前にプロセスが終了した場合 `exited` が発火し `state` が変更されないことを確認する
- [x] 3.2 `spawn_supervised_for_startup` の単体テスト: `confirm` 送信後にプロセスが終了した場合、現行同様 `state.set_error()` が呼ばれることを確認する(既存の `crash_updates_state_to_disconnected` を参考にする)
- [x] 3.3 `start_backend` のリトライループについて、ポート競合を模したシナリオ(1回目は即終了するダミーコマンド、2回目以降は正常起動)で最終的に成功することを検証するテストを追加する(既存テストのモック手法を踏襲する)

## 4. ドキュメント・確認

- [x] 4.1 `cargo test` / `cargo clippy --all-targets -- -D warnings` が通ることを確認する

## 5. コードレビューで判明した追加修正(実装後)

- [x] 5.1 リトライ中(起動確認待機中)にアプリを終了しても `bw serve` 子プロセスが `ManagedProcess` に登録されずkillされない問題を修正(`register_process` を各試行のspawn直後に呼ぶ)
- [x] 5.2 `state.set_port(port)` が `readiness_check` 完了後(=ロック状態セット後)に呼ばれており不整合ウィンドウが生じる問題を修正(spawn直後に呼ぶ)
- [x] 5.3 全試行失敗後も `state.port()` に最後の(死んだ)ポート番号が残る問題を修正(`AppState::set_error()` が `port` も同時にクリアするようにした)
- [x] 5.4 `process.rs` の監視ロジック(`spawn_supervised_with_command` と confirm後の分岐)の重複を `supervise_until_exit` に共通化
- [x] 5.5 リトライ導入で不要になった `spawn_supervised` / `spawn_supervised_with_command` / `spawn_supervised_for_startup`(いずれも `_with_command` 以外)のdead codeを削除
- [x] 5.6 起動確認待機中のアプリ終了(`ProcessHandle::shutdown()`)をクラッシュと誤認してリトライしてしまう問題を修正(`StartupExit::Crashed`/`ShutdownRequested` を導入して区別)
- [x] 5.7 readiness_check成功とbw serveのクラッシュがほぼ同時に起きた場合に、死んだプロセスを「成功」として扱ってしまう可能性のあるレースを緩和(外側selectを`biased`化 + `confirm`送信直前に`exited`を非ブロッキングで再確認)。完全には閉じられないナノ秒オーダーの残存リスクをdesign.mdに明記

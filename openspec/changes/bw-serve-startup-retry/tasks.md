## 1. process.rs: 2フェーズ監視の追加

- [ ] 1.1 `process.rs` に `MAX_STARTUP_ATTEMPTS`(=3)定数を追加する
- [ ] 1.2 `StartupHandles`(`process_handle`, `monitor`, `exited: oneshot::Receiver<()>`, `confirm: oneshot::Sender<()>`)を新設する
- [ ] 1.3 `spawn_supervised_for_startup(port, state)` を新設する。監視タスクを「`confirm` 受信前は exit を `exited` チャンネルへ通知するだけで `state` には触れない」「`confirm` 受信後は現行の `spawn_supervised_with_command` と同じ『予期せぬ終了→`state.set_error()`』監視に切り替える」の2フェーズ構造にする
- [ ] 1.4 既存の `spawn_supervised` / `spawn_supervised_with_command` は変更せず温存し、`build_bw_serve_command` 等の共通パーツを `spawn_supervised_for_startup` からも再利用する

## 2. main.rs: リトライループの実装

- [ ] 2.1 `start_backend` 内の「`pick_free_port()` → `spawn_supervised()` → `state.set_port()` → `sync_initial_status()`」を、`MAX_STARTUP_ATTEMPTS` 回までのリトライループに置き換える
- [ ] 2.2 各試行で `tokio::select!` により `sync_initial_status(&client, &state)` の完了と `handles.exited` を競合させる。`exited` が先に発火した場合は `eprintln!` でログを出しつつ次の試行(新しいポート)へ進む
- [ ] 2.3 `sync_initial_status` が先に完了した場合は `handles.confirm` を送信し、`ManagedProcess` への登録と `state.set_port(port)` を行って成功終了する
- [ ] 2.4 全試行が早期終了で尽きた場合、`state.set_error(...)` でエラー状態を記録して終了する

## 3. テスト

- [ ] 3.1 `spawn_supervised_for_startup` の単体テスト: `confirm` 送信前にプロセスが終了した場合 `exited` が発火し `state` が変更されないことを確認する
- [ ] 3.2 `spawn_supervised_for_startup` の単体テスト: `confirm` 送信後にプロセスが終了した場合、現行同様 `state.set_error()` が呼ばれることを確認する(既存の `crash_updates_state_to_disconnected` を参考にする)
- [ ] 3.3 `start_backend` のリトライループについて、ポート競合を模したシナリオ(1回目は即終了するダミーコマンド、2回目以降は正常起動)で最終的に成功することを検証するテストを追加する(既存テストのモック手法を踏襲する)

## 4. ドキュメント・確認

- [ ] 4.1 `cargo test` / `cargo clippy --all-targets -- -D warnings` が通ることを確認する

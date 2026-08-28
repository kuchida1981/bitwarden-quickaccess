## Why

`pick_free_port()` は一時的にTCP bindしてポート番号を取得し、すぐ解放してから `bw serve --port` に渡している。このbind→解放→起動の間(TOCTOU)に他プロセスが同じポートを奪う可能性がゼロではない。現状ポート取得は起動時に1回だけで、`bw serve` が起動直後にポート競合等で異常終了した場合のリトライ機構がないため、ユーザーにエラー画面が表示され、アプリの再起動が必要になってしまう(GitHub issue #119)。

## What Changes

- `bw serve` 起動直後のグレース期間内(2秒)にプロセスが異常終了した場合、原因を問わず「起動失敗」とみなし、ポートを再取得して自動的に再試行する仕組みを追加する
- リトライは最大3回(初回込み)まで行い、上限に達した場合は現行通り `state.set_error()` でエラー状態を記録する
- グレース期間を過ぎて安定稼働に入った後は、現行の監視タスク(予期せぬ終了を検知して `state.set_error()` する)にそのまま引き継ぐ
- リトライの発生・結果を `eprintln!` でログ出力する(構造化ロギング基盤 issue #84 は未着手のため、既存パターンを踏襲)

## Capabilities

### New Capabilities

なし

### Modified Capabilities

- `vault-backend-service`: 「bw serveプロセスのライフサイクル管理」要件に、起動直後の異常終了時はポートを再取得して自動リトライする、というふるまいを追加する

## Impact

- `app/src-tauri/src/backend/process.rs`(`spawn_supervised` まわりに新しいリトライ付き起動関数を追加)
- `app/src-tauri/src/main.rs`(`start_backend` の呼び出し箇所を新関数に置き換え)
- 既存の `spawn_supervised` / `spawn_supervised_with_command` はテスト容易性のため温存しつつ、上位のリトライロジックから利用する形にする想定(design.mdで確定させる)

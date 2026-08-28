## Why

`bw serve` の起動引数に `--hostname localhost` を渡しているが、`localhost` の名前解決はOS/ネットワーク設定次第でIPv6(`::1`)にバインドされる場合がある。一方、ポート番号を確保する `pick_free_port()` はIPv4ループバック(`127.0.0.1`)にbindしており、クライアント(`BwServeClient`)も `127.0.0.1:{port}` を前提に通信する。この不一致があると、起動確認や通信が失敗するリスクがある(GitHub issue #82)。

## What Changes

- `process.rs` の `build_bw_serve_command` で `bw serve` の起動引数を `--hostname localhost` から `--hostname 127.0.0.1` に変更する
- `pick_free_port` / `build_bw_serve_command` のdocコメントを実態(IPv4ループバックへの明示バインド)に合わせて更新する
- `build_bw_serve_command` が組み立てる引数を検証する軽量なテストを追加する

## Capabilities

### New Capabilities

なし

### Modified Capabilities

- `vault-backend-service`: 「動的ポート割り当てとlocalhost限定バインド」要件の記述を、`--hostname localhost` から `--hostname 127.0.0.1` を明示指定する内容に更新する(バインド先を外部に晒さないという要件自体は変わらない)

## Impact

- `app/src-tauri/src/backend/process.rs`(`build_bw_serve_command`)
- `openspec/specs/vault-backend-service/spec.md`
- 既存の起動フロー・テストへの互換性影響なし(127.0.0.1はIPv4ループバックであり、既存のポート確保ロジックと整合する)

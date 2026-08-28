## Why

`bw serve` の起動引数に `--hostname localhost` を渡しているが、`localhost` の名前解決はOS/ネットワーク設定次第でIPv6(`::1`)にバインドされる場合がある。一方、ポート番号を確保する `pick_free_port()` はIPv4ループバック(`127.0.0.1`)にbindしている。さらに実装時に判明したこととして、クライアント(`BwServeClient::new`)も `http://localhost:{port}` を使っており、当初の想定(既に`127.0.0.1`を使っている)と異なり実際にはサーバー・クライアント両方が`localhost`依存だった。この不一致があると、起動確認や通信が失敗するリスクがある(GitHub issue #82)。

## What Changes

- `process.rs` の `build_bw_serve_command` で `bw serve` の起動引数を `--hostname localhost` から `--hostname 127.0.0.1` に変更する
- `http_client.rs` の `BwServeClient::new` で組み立てるbase URLを `http://localhost:{port}` から `http://127.0.0.1:{port}` に変更する(実装時に、サーバー側だけでなくクライアント側も`localhost`依存だったことが判明したため追加)
- `pick_free_port` / `build_bw_serve_command` / `BwServeClient::new` のdocコメントを実態(IPv4ループバックへの明示バインド)に合わせて更新する
- 上記2箇所それぞれについて、実際に組み立てられる引数・接続先を検証する軽量なテストを追加する

## Capabilities

### New Capabilities

なし

### Modified Capabilities

- `vault-backend-service`: 「動的ポート割り当てとlocalhost限定バインド」要件の記述を、`--hostname localhost` から `--hostname 127.0.0.1` を明示指定する内容に更新する(バインド先を外部に晒さないという要件自体は変わらない)

## Impact

- `app/src-tauri/src/backend/process.rs`(`build_bw_serve_command`)
- `app/src-tauri/src/backend/http_client.rs`(`BwServeClient::new`)
- `openspec/specs/vault-backend-service/spec.md`
- 既存の起動フロー・テストへの互換性影響なし(127.0.0.1はIPv4ループバックであり、既存のポート確保ロジックと整合する)

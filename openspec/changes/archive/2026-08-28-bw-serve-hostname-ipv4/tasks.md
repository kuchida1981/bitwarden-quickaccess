## 1. 実装

- [x] 1.1 `app/src-tauri/src/backend/process.rs` の `build_bw_serve_command` で `--hostname localhost` を `--hostname 127.0.0.1` に変更する
- [x] 1.2 `pick_free_port` / `build_bw_serve_command` のdocコメントを実態(IPv4ループバックへの明示バインド)に合わせて更新する
- [x] 1.3 `build_bw_serve_command` が組み立てる引数(`--hostname 127.0.0.1` を含むこと)を検証するユニットテストを追加する

## 2. ドキュメント

- [x] 2.1 `cargo test` / `cargo clippy --all-targets -- -D warnings` が通ることを確認する

## 3. コードレビューで判明した追加修正

- [x] 3.1 `app/src-tauri/src/backend/http_client.rs` の `BwServeClient::new` が `http://localhost:{port}` を使っており、サーバー側だけ`127.0.0.1`に固定してもクライアント側の名前解決不一致が解消されないことが判明したため、`http://127.0.0.1:{port}` に修正する
- [x] 3.2 `BwServeClient::new` が実際に127.0.0.1へ接続することを検証する回帰テストを追加する

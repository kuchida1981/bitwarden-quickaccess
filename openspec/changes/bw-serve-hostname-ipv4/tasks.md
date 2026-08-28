## 1. 実装

- [ ] 1.1 `app/src-tauri/src/backend/process.rs` の `build_bw_serve_command` で `--hostname localhost` を `--hostname 127.0.0.1` に変更する
- [ ] 1.2 `pick_free_port` / `build_bw_serve_command` のdocコメントを実態(IPv4ループバックへの明示バインド)に合わせて更新する
- [ ] 1.3 `build_bw_serve_command` が組み立てる引数(`--hostname 127.0.0.1` を含むこと)を検証するユニットテストを追加する

## 2. ドキュメント

- [ ] 2.1 `cargo test` / `cargo clippy --all-targets -- -D warnings` が通ることを確認する

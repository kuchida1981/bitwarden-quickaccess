## Context

`process.rs::build_bw_serve_command` は `bw serve --hostname localhost --port <port>` として子プロセスを起動している。`pick_free_port()` は `TcpListener::bind(("127.0.0.1", 0))` でIPv4ループバックにbindしてポート番号だけを取得する。

**実装時の訂正**: 当初この設計では `http_client::BwServeClient` が既に `http://127.0.0.1:{port}` にリクエストしていると想定していたが、実際には `BwServeClient::new` も `http://localhost:{port}` を使っていた(コードを確認せずに書いた誤った前提)。つまりサーバー(`build_bw_serve_command`)・クライアント(`BwServeClient::new`)の両方が `localhost` の名前解決に依存しており、`pick_free_port` だけがIPv4リテラルを直接使っていた。`localhost` の名前解決結果はOS設定(`/etc/hosts` や `getaddrinfo` の優先順位)に依存し、環境によってはIPv6(`::1`)が優先されうる。サーバー側だけを `127.0.0.1` に固定してクライアント側を放置すると、IPv6環境ではむしろ「サーバーはIPv4のみ・クライアントはIPv6へ」という確実な不一致を生んでしまうため、Claude Codeによるコードレビューでこの誤りが指摘され、`BwServeClient::new` も合わせて修正した。

## Goals / Non-Goals

**Goals:**
- `bw serve` のバインド先を、ポート確保・クライアント接続と同じIPv4ループバック(`127.0.0.1`)に統一する

**Non-Goals:**
- IPv6環境そのもののサポート
- ポート確保のTOCTOU(bind→解放→起動の間に奪われるリスク)への対処(別issue #119で扱う、`bw-serve-startup-retry` で対応済み)

## Decisions

- **`--hostname 127.0.0.1` を明示指定し、`BwServeClient::new` のbase URLも `http://127.0.0.1:{port}` に統一する**: `localhost` という名前解決に依存する記述をサーバー・クライアント双方からなくし、`pick_free_port` と同じIPv4リテラルを直接使う。名前解決の余地を無くすことで環境差によるIPv4/IPv6不一致を構造的に排除できる。IPv6アドレス(`::1`)へ変更する代替案もあるが、`pick_free_port` が既にIPv4前提であり変更範囲が広がるため採用しない。

## Risks / Trade-offs

- [Risk] 将来IPv6環境への対応が必要になった場合、`pick_free_port`・`BwServeClient::new`・`build_bw_serve_command` の3箇所を同時に変更する必要がある → Mitigation: 今回の変更でこの3箇所を`127.0.0.1`前提に統一したことで、今後はむしろ一貫性が保たれる(以前はサーバー/クライアントが`localhost`、ポート確保だけがIPv4という不揃いな状態だった)

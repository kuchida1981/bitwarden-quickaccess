## Context

`process.rs::build_bw_serve_command` は `bw serve --hostname localhost --port <port>` として子プロセスを起動している。`pick_free_port()` は `TcpListener::bind(("127.0.0.1", 0))` でIPv4ループバックにbindしてポート番号だけを取得し、`http_client::BwServeClient` も `http://127.0.0.1:{port}` にリクエストする。`localhost` の名前解決結果はOS設定(`/etc/hosts` や `getaddrinfo` の優先順位)に依存し、環境によってはIPv6(`::1`)が優先されうる。

## Goals / Non-Goals

**Goals:**
- `bw serve` のバインド先を、ポート確保・クライアント接続と同じIPv4ループバック(`127.0.0.1`)に統一する

**Non-Goals:**
- IPv6環境そのもののサポート
- ポート確保のTOCTOU(bind→解放→起動の間に奪われるリスク)への対処(別issue #119で扱う)

## Decisions

- **`--hostname 127.0.0.1` を明示指定する**: `localhost` という名前解決に依存する記述をやめ、`pick_free_port`/`BwServeClient` と同じIPv4リテラルを直接渡す。名前解決の余地を無くすことで環境差によるIPv4/IPv6不一致を構造的に排除できる。IPv6アドレス(`::1`)へ変更する代替案もあるが、既存のポート確保・クライアント実装が既にIPv4前提であり変更範囲が広がるため採用しない。

## Risks / Trade-offs

- [Risk] 将来IPv6環境への対応が必要になった場合、`pick_free_port`・`BwServeClient`・`build_bw_serve_command` の3箇所を同時に変更する必要がある → Mitigation: 現状もこの3箇所はIPv4前提で密結合しており、今回の変更で新たな結合を生むものではない

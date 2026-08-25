## REMOVED Requirements

### Requirement: session tokenのOSキーチェーンへのキャッシュ
**Reason**: TUI(`bin/bw-quickaccess`)廃止に伴い、session tokenをOSキーチェーンにキャッシュする方式は使われなくなる。
**Migration**: GUIアプリでは `bw-serve-backend` change の `vault-backend-service` capability が、`bw serve` プロセスにアンロック状態をメモリ上で保持する方式に置き換わる。

### Requirement: TTLによるセッション有効期限管理
**Reason**: TUI固有のTTLベースのセッション管理は使われなくなる。
**Migration**: 同等の考え方(既定15分)は `credential-actions-autolock` change の `idle-auto-lock` capability に引き継がれる。

### Requirement: セッション無効時の実利用時フォールバック
**Reason**: TUIのサブプロセス実行時における再認証フォールバックの仕組みは使われなくなる。
**Migration**: GUIアプリでは `vault-unlock-prompt` capability(`quickaccess-search-ui` change)がロック検知時のアンロックUIを提供する。

### Requirement: session tokenをコマンドライン引数として渡さない
**Reason**: `bw` コマンドをコマンドライン引数付きで都度起動する実行方式自体が廃止される。
**Migration**: GUIアプリは `bw serve` のHTTP API経由でアクセスし、session tokenをコマンドライン引数として扱う箇所がそもそも存在しない(`vault-backend-service` capability)。

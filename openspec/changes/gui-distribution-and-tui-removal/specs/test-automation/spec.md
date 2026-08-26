## REMOVED Requirements

### Requirement: 純粋ロジック・分岐ロジックの単体テスト
**Reason**: `lib/*.sh` に対するbatsベースの単体テストは、TUIコード自体の削除に伴い廃止される。
**Migration**: Rust/Tauriコードベースに対するテスト戦略(`cargo test` 等)は実装時に別途定める。

### Requirement: 外部コマンド依存箇所のモック方式
**Reason**: bashスクリプトにおける外部コマンドのモック手法はTUI廃止に伴い廃止される。
**Migration**: Rust側でのモック方式(`bw serve` を使ったテスト用ヘルパー等)は各changeのtasks.mdで個別に定める。

### Requirement: CI による構文チェック・静的解析・テスト実行の自動化
**Reason**: `bash -n`/`shellcheck`/`bats` によるCIはTUI廃止に伴い廃止される。
**Migration**: `cargo build`/`cargo test` 等によるCIに置き換わる(本change design.md の Decision 4 参照)。

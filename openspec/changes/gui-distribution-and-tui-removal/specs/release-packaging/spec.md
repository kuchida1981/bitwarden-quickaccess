## REMOVED Requirements

### Requirement: 単一ファイルへのバンドル
**Reason**: `bin/bw-quickaccess` と `lib/*.sh` を単一ファイルに結合するTUI固有のバンドル方式はTUI廃止に伴い廃止される。
**Migration**: GUIアプリは `tauri build` が生成する `.app` バンドルに置き換わる(`gui-distribution` capability)。

### Requirement: リリース公開時のアセット自動添付
**Reason**: TUIの単一ファイル成果物をリリースに添付する既存ロジックはTUI廃止に伴い廃止される。
**Migration**: GUIアプリの `.app` を未署名のままリリースアセットとして自動添付する仕組みに置き換わる(`gui-distribution` capability)。

### Requirement: バージョンの埋め込みとバージョン確認コマンド
**Reason**: `bw-quickaccess --version` によるバージョン確認コマンドはTUI廃止に伴い廃止される。
**Migration**: GUIアプリは `tauri.conf.json`/`Cargo.toml` のversionフィールドを単一の情報源とし、アプリ内メニューからバージョンを確認できるようにする(`gui-distribution-and-tui-removal` change の design.md 参照)。

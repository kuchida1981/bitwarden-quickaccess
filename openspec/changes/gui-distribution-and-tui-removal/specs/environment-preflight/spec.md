## REMOVED Requirements

### Requirement: 必須外部コマンドの存在確認
**Reason**: `jq`/`fzf` 等、TUI実行に必要だった外部コマンド群への依存がGUIアプリでは無くなる。
**Migration**: GUIアプリが唯一依存する外部コマンドは `bw` CLIのみであり、その存在確認は `vault-backend-service` capability(`bw-serve-backend` change)の「bw serve CLI 前提チェック」要件が担う。

### Requirement: クリップボードコピーコマンドの検出と確認
**Reason**: `pbcopy`/`wl-copy`/`xclip` 等のOSクリップボードコマンドへの依存はGUIアプリでは無くなる(OSネイティブのクリップボードAPIを直接使用する)。
**Migration**: 対応する事前チェックは不要になる。

### Requirement: keychain連携ツールの疎通確認
**Reason**: `security`/`secret-tool` を介したkeychain連携はTUI固有の実装であり、GUIアプリでは `bw serve` がメモリ上にアンロック状態を保持する方式に置き換わるため不要になる。
**Migration**: 該当なし(`vault-backend-service` capability が新しいセッション保持方式を定める)。

### Requirement: fzfの最低バージョン確認
**Reason**: `fzf` 依存自体がGUIアプリでは無くなる。
**Migration**: 該当なし。

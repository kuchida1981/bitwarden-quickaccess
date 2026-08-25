## REMOVED Requirements

### Requirement: フィールド選択によるクリップボードコピー
**Reason**: fzfのフィールド選択画面(2段階選択UI)はTUI廃止に伴い使われなくなる。
**Migration**: GUIアプリでは検索結果一覧の行に対して直接ショートカットキーでコピーする(`credential-copy-actions` capability, `credential-actions-autolock` change)。

### Requirement: キーバインドによるショートカットコピー
**Reason**: fzfの `ctrl-o`/`ctrl-r`/`ctrl-t` によるショートカットコピーはTUI廃止に伴い使われなくなる。
**Migration**: GUIアプリでは `⌘C`/`⌘⇧C`/`⌥⌘C` によるショートカットコピーが `credential-copy-actions` capability として引き継ぐ。

### Requirement: 連続コピー
**Reason**: フィールド選択画面を開いたままにして複数フィールドを連続コピーするTUI固有のUIは廃止される。
**Migration**: GUIアプリでは1回のショートカット操作で1フィールドをコピーし、ポップアップを閉じる設計に変わる(`credential-actions-autolock` change の設計判断による)。

### Requirement: 機密情報を標準出力に出力しない
**Reason**: 標準出力を経由する実行方式自体がTUI廃止に伴い使われなくなる。
**Migration**: GUIアプリでは秘匿値をRustコア内で完結させ、WebView側JSに渡さない設計を `credential-actions-autolock` change の design.md で定めている。

### Requirement: フィールド選択画面の表示順序
**Reason**: フィールド選択画面自体がTUI廃止に伴い廃止される。
**Migration**: GUIアプリの検索結果一覧の表示順序は `incremental-item-search` capability(`quickaccess-search-ui` change)が定める。

### Requirement: フィールド選択画面のフルスクリーン表示
**Reason**: fzfのフルスクリーン(alternate screen buffer)表示はTUI廃止に伴い使われなくなる。
**Migration**: GUIアプリはOSネイティブのポップアップウィンドウとして表示される(`global-hotkey-popup` capability, `menubar-hotkey-shell` change)。

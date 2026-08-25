## REMOVED Requirements

### Requirement: Vaultアイテムのインクリメンタルサーチ
**Reason**: fzfベースのインクリメンタルサーチ画面はTUI廃止に伴い廃止される。
**Migration**: GUIアプリの検索ボックス+一覧が `incremental-item-search` capability(`quickaccess-search-ui` change)として引き継ぐ。

### Requirement: 直前選択アイテムの記憶による検索スキップ
**Reason**: 次回起動時に直前のフィールド選択画面から始めるTUI固有の挙動は、GUIアプリの起動体験(毎回検索画面から始まる)には引き継がれない。
**Migration**: 該当なし(GUIアプリはホットキー押下のたびに検索画面から始まる設計とする)。

### Requirement: 検索画面でのキーバインドによる直接コピー
**Reason**: fzf検索画面上での `ctrl-r`/`ctrl-o`/`ctrl-t` 直接コピーはTUI廃止に伴い廃止される。
**Migration**: GUIアプリの一覧行に対するショートカットコピー(`credential-copy-actions` capability, `credential-actions-autolock` change)に置き換わる。

### Requirement: 検索画面のフルスクリーン表示
**Reason**: fzfのフルスクリーン(alternate screen buffer)表示はTUI廃止に伴い廃止される。
**Migration**: GUIアプリはOSネイティブのポップアップウィンドウとして表示される(`global-hotkey-popup` capability, `menubar-hotkey-shell` change)。

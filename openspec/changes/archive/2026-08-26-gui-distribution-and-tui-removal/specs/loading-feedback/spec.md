## REMOVED Requirements

### Requirement: bw CLI呼び出し中のローディングメッセージ表示
**Reason**: TUIの `bwqa_log` による1行メッセージ表示はTUI廃止に伴い使われなくなる。
**Migration**: GUIアプリでは `vault-unlock-prompt` capability(`quickaccess-search-ui` change)がアンロック処理中・結果のUI表示を担う。処理中のローディング表現の要否は実装時に別途検討する。

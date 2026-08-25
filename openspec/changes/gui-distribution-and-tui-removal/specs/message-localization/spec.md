## REMOVED Requirements

### Requirement: 言語ファイルによるメッセージ管理
**Reason**: `lib/i18n/*.sh` によるTUIメッセージの言語ファイル管理はTUI廃止に伴い廃止される。
**Migration**: GUIアプリのローカライズはv1.0.0のスコープ外(non-goal)とする。必要になれば将来のchangeで改めて検討する。

### Requirement: 言語の自動判定と明示指定
**Reason**: `LANG`/`LC_ALL`/`BWQA_LANG` によるTUIの言語自動判定・明示指定はTUI廃止に伴い廃止される。
**Migration**: 該当なし(v1.0.0では単一言語のUIとする)。

### Requirement: 既存メッセージ出力箇所の網羅的な移行
**Reason**: TUIの全メッセージ出力箇所を対象とした移行要件は、TUI自体の廃止に伴い意味を失う。
**Migration**: 該当なし。

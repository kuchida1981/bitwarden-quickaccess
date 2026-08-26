## REMOVED Requirements

### Requirement: コピー結果のフィードバック表示
**Reason**: fzfのボーダーラベル書き換えによるフィードバック表示はTUI廃止に伴い使われなくなる。
**Migration**: GUIアプリでは `credential-actions-autolock` change の `credential-copy-actions` capability がコピー結果フィードバックを提供する。

### Requirement: コピー処理中の進行状況表示
**Reason**: fzfの `every(N)` + `bg-transform-border-label` によるスピナー表示の仕組みはTUI廃止に伴い使われなくなる。
**Migration**: GUIアプリでのコピー処理は `bw serve` へのHTTP呼び出しであり、体感速度・進行表示の要否は実装時に別途検討する(v1.0.0では即時実行を前提とし、専用のローディング表示は設けない)。

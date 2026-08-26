## Context

Escapeキーの処理は現在3箇所に分散している(いずれも `app/dist/app.js`):
- `searchBox` のkeydownリスナー内(`helpOpen`/`actionMenuOpen` の状態を見て、通常時は `hide_popup` を呼ぶ)
- `handleActionMenuKeydown`(`ArrowLeft` と共に `Escape` でアクションメニューを閉じる)
- `handleHelpKeydown`(`⌘/` と共に `Escape` でヘルプオーバーレイを閉じる)

これらはすべて `searchBox` にフォーカスがある場合にしか発火しない。アンロック画面の `passwordInput`/`unlockForm` にフォーカスがある間はこの経路に到達しないため、Escapeキーが機能しない(issue #76)。

`quickaccess-help-escape` spec の「Escapeキーによるポップアップのクローズ」要件は画面を限定していないため、実装をこの要件に合わせる。

## Goals / Non-Goals

**Goals:**
- どの要素にフォーカスがあってもEscapeキーが一貫して機能するようにする。
- ヘルプオーバーレイ表示中・アクションメニュー表示中の優先順位(ヘルプ > アクションメニュー > 通常のポップアップクローズ)は現状を維持する。
- 実装を単純化し、将来画面が増えても(例: issue #79の「bw未検出」エラー画面)個別にEscapeリスナーを追加する必要がないようにする。

**Non-Goals:**
- ヘルプ・アクションメニュー自体の仕様変更。
- Escape以外のキー処理(検索ボックス固有のショートカット等)の変更。

## Decisions

- **Escapeキーの処理を `document` レベルの単一リスナーに集約する**。優先順位判定(ヘルプ > アクションメニュー > 通常)はこの1箇所にまとめ、`searchBox` のkeydownリスナー、`handleActionMenuKeydown`、`handleHelpKeydown` からはEscape固有の分岐を削除する。
  - 理由: フォーカス要素に依存しない一貫した挙動が得られ、今後画面が増えても個別対応が不要になる(design上の代替案として「`unlockForm`/`passwordInput` にも同じリスナーを個別追加する」を検討したが、画面が増えるたびに同じ実装を複製することになり保守性が低いため採用しなかった)。
- **`event.preventDefault()` は集約後のリスナー内で1回のみ行う**。各画面固有のキー処理関数(`handleActionMenuKeydown` 等)は他のキー(Arrow等)の処理を引き続き担当するため、Escape以外の分岐はそのまま残す。

## Risks / Trade-offs

- [`document` レベルのリスナーに変更することで、意図しない要素(例: 将来追加されるテキスト入力欄)でもEscapeがグローバルに反応してしまう可能性] → 現状の画面構成(検索・アンロックのみ)では問題にならない。将来的にモーダル等が増えた場合は、その時点で優先順位分岐に追加する。
- [リスナーの集約により、既存のヘルプ/アクションメニューのEscape優先順位を壊すリグレッションの可能性] → 動作確認タスクで既存の優先順位(ヘルプ>アクションメニュー>通常)が壊れていないことを確認する。

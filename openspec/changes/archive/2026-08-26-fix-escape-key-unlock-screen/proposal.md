## Why

GitHub issue #76: Vaultがロックされている状態(マスターパスワード入力画面)でクイックアクセスを開いた場合、`Esc` キーを押してもポップアップが閉じない。既存の `quickaccess-help-escape` spec の「Escapeキーによるポップアップのクローズ」要件は「検索ポップアップが表示されている状態で、ヘルプもアクションメニューも表示されていないとき」と画面を限定せずに定義されており、この要件自体は正しい。実装がアンロック画面をカバーしていないだけの純粋なバグである。

原因: `app/dist/app.js` のEscapeキー処理が `searchBox.addEventListener("keydown", ...)` にのみ登録されており、アンロック画面の `passwordInput`/`unlockForm` にフォーカスがある間はこのリスナーに到達しない。

## What Changes

- Escapeキーの処理(ポップアップを閉じる/ヘルプを閉じる/アクションメニューを閉じる、の優先順位判定を含む)を、`searchBox` にスコープした個別リスナーから `document` レベルの単一リスナーに集約する。
- これにより、フォーカスがどの要素(検索ボックス・パスワード入力欄・将来追加されうる他の画面)にあってもEscapeキーが一貫して機能するようになる。
- 既存の `searchBox` のkeydownリスナー内、`handleActionMenuKeydown`、`handleHelpKeydown` からEscape固有の分岐を削除し、二重処理を避ける。

## Capabilities

### New Capabilities
(なし)

### Modified Capabilities
(なし。`quickaccess-help-escape` の既存要件はそのままで、実装をその要件に適合させるバグ修正のため spec の変更は不要)

## Impact

- `app/dist/app.js`: Escapeキー処理の集約(`searchBox` のkeydownリスナー、`handleActionMenuKeydown`、`handleHelpKeydown`)
- 破壊的変更なし。既存のEscape挙動(検索画面・ヘルプ・アクションメニュー)は変わらず、アンロック画面でも同様に機能するようになる。

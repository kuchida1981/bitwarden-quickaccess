## Why

検索結果一覧上にマウスカーソルを置いたまま↓/↑キーで行フォーカスを移動すると、一覧のスクロールが発生する境目でフォーカスがキー操作した行ではなく「マウスカーソル下の行」に奪われてしまう(#128)。誤って別アイテムをコピー/操作してしまうリスクがあり、既存の「フォーカス行移動の安定性」要件(`incremental-item-search`)が意図する安定動作を損なっている。

## What Changes

- `app/dist/app.js` の `scrollIntoView` 呼び出し(`renderResults()` および `updateFocusRows()` 内)を起点に発生する、スクロールによる意図しない `mouseenter` 再発火を無視する仕組みを追加する
- 実際のマウス移動(`mousemove`)が観測されるまでは、プログラム的スクロール直後の `mouseenter` によるフォーカス上書きを抑止する

## Capabilities

### New Capabilities

(なし)

### Modified Capabilities

- `incremental-item-search`: 「フォーカス行移動の安定性」要件に、スクロールを伴う行フォーカス移動時のシナリオを追加する(既存要件の意図はそのままに、スクロール起因の`mouseenter`誤爆という未カバーの経路を明記する)

## Impact

- 影響ファイル: `app/dist/app.js`(`renderResults`, `updateFocusRows`, 各行の `mouseenter` リスナー)
- HTML/CSSの変更は不要
- 関連issue #111(検索ボックスのサジェスト表示)は別changeで扱う
- 関連issue #91(イベントデリゲーション改善)とは別経路の不具合であり、本changeのスコープ外とする

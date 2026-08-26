## Why

GitHub issue #65(アクション実行後のフィードバックメッセージの残留バグ)の調査中、テキストによるフィードバック表示そのものの必要性を再検討した。現状、コピー等のアクション成功時は「メッセージ表示」と「700ms後にクイックアクセスが閉じる」という**2つの成功シグナル**を同時に出しており冗長である。またキーボードショートカット(`⌘C`/`⌘⇧C`/`⌥⌘C`/`Enter`)は対象フィールドの有無を事前チェックしないため、失敗時にテキスト以外の手段でユーザーに結果を伝える方法がなく、このテキスト状態の管理(表示・クリア)が複雑化し、残留バグの温床になっていた。

1Password Quick Accessに倣い、「フォーカス行の点滅」で入力受付を伝え、「クイックアクセスの開閉」で成功/失敗を伝える設計に切り替えることで、テキスト状態そのものを無くし、残留バグを構造的に解消する。

## What Changes

- コピー(ユーザー名/パスワード/TOTP)およびブラウザ起動アクションについて、テキストによる結果フィードバック(`showFeedback`, `#feedback` 要素、関連i18n文字列)を廃止する。
- 代わりに、アクション実行時にフォーカス中の行を点滅させることで「入力が受け付けられた」ことを常に(成功・失敗によらず)示す。
- 点滅の後、アクションが**成功した場合はクイックアクセスを閉じ**、**失敗した場合は閉じずに検索画面のまま留まる**ことで成功/失敗を伝える。
- 上記はショートカット直押し・アクションメニュー経由のいずれの起動方法でも同一の `executeItemAction`/`runAction` を通るため、単一の実装で両方に適用される。
- 不要になった i18n 文字列(`copiedUsername` / `copiedPassword` / `copiedTotp` / `openedInBrowser` / `actionFailed`)と `#feedback` 関連のDOM・CSSを削除する。

## Capabilities

### New Capabilities
(なし)

### Modified Capabilities
- `credential-copy-actions`: 「コピー結果フィードバック」要件を、テキスト表示から「行の点滅+開閉による視覚フィードバック」に変更する。
- `open-in-browser-action`: 「Enterキーによるブラウザ起動」「URL未設定時のフィードバック」の両要件に、行の点滅+開閉による視覚フィードバックを反映する。
- `ui-localization`: 「検索ポップアップUI文言のローカライズ」要件から、廃止されるコピー操作結果テキストに関する記述・シナリオを削除する。

## Impact

- `app/dist/app.js`: `showFeedback`, `runAction`, `executeItemAction`, `renderResults`(点滅アニメーション用のクラス付与)
- `app/dist/index.html`: `#feedback` 要素の削除(または未使用化)
- `app/dist/style.css`: `#feedback` 関連スタイルの削除、行点滅アニメーション用スタイルの追加
- `app/dist/i18n.js`: `copiedUsername` / `copiedPassword` / `copiedTotp` / `openedInBrowser` / `actionFailed` の削除
- `app/src-tauri/src/popup.rs`, `commands.rs`: 本redesignにより、issue #65が問題としていた「非表示イベントに依存したフィードバック管理」自体が不要になるため、当初想定していたイベント配線の変更は不要になる見込み(詳細はdesign.md参照)。
- 破壊的変更なし(ユーザー向けの見た目が変わるのみ。外部API・設定ファイルへの影響なし)。

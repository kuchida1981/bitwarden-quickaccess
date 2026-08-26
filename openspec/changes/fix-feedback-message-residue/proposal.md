## Why

コピー等のアクション実行後に表示されるフィードバックメッセージ(例:「ユーザー名をコピーしました」)が、クイックアクセスを閉じてすぐに再度開いた際、クリアされずに残留することがある。フィードバックのクリア処理が「非表示→表示」の画面遷移(`POPUP_SHOWN_EVENT`)にしか紐付いておらず、素早いトグル操作でこの遷移がうまく起きないケースがあるため。ユーザーに古い操作結果を誤って提示してしまう体験上のバグであり、GitHub issue #65 に対応する。

## What Changes

- フィードバック表示に独立した自動消去タイムアウト(2〜3秒程度)を設け、表示から一定時間後は必ずメッセージが消えるようにする。
- ポップアップが非表示になるタイミング(`hide_popup` コマンド呼び出し時、および `Focused(false)` によるフォーカス喪失時の自動hide)で、フロントエンド側のフィードバック状態を確実にリセットする。
- 上記により、`POPUP_SHOWN_EVENT` の発火有無に依存せずフィードバックが残留しないことを保証する。

## Capabilities

### New Capabilities
(なし)

### Modified Capabilities
- `credential-copy-actions`: 「コピー結果フィードバック」要件に、フィードバックの消去タイミング(独立タイムアウトによる自動消去、およびポップアップ非表示時のリセット)に関する規定を追加する。

## Impact

- `app/dist/app.js`: `showFeedback`, `runAction`, `handleShown` まわりのフィードバック表示・クリアロジック
- `app/src-tauri/src/popup.rs`: `toggle_popup`(非表示分岐)
- `app/src-tauri/src/commands.rs`: `hide_popup`
- 破壊的変更なし。既存のコピー成功/失敗フィードバックの表示内容自体は変更しない。

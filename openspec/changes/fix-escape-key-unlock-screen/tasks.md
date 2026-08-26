## 1. Escapeキー処理の集約

- [x] 1.1 `app/dist/app.js` に、`document` レベルの `keydown` リスナーを追加する。優先順位は「ヘルプオーバーレイ表示中(`helpOpen`)→ ヘルプを閉じる」「アクションメニュー表示中(`actionMenuOpen`)→ アクションメニューを閉じる」「それ以外 → `invoke("hide_popup")`」の順とする。
- [x] 1.2 `searchBox` のkeydownリスナー内(`app.js` 内の `if (event.key === "Escape") { ... invoke("hide_popup") ... }` のブロック)から、Escape固有の分岐を削除する(1.1に処理が移るため)。
- [x] 1.3 `handleActionMenuKeydown` 内の `if (event.key === "ArrowLeft" || event.key === "Escape")` から `event.key === "Escape"` の条件を外し、`ArrowLeft` のみでアクションメニューを閉じる分岐として残す(Escapeは1.1の集約リスナーが処理する)。
- [x] 1.4 `handleHelpKeydown` 内の `if (event.key === "Escape" || isHelpToggleShortcut(event))` から `event.key === "Escape" ||` を外し、`isHelpToggleShortcut(event)` のみでヘルプを閉じる分岐として残す(Escapeは1.1の集約リスナーが処理する)。

## 2. 動作確認

- [x] 2.1 `cargo test` を実行し、既存テストが通ることを確認する。(2026-08-26 実行、全35テスト成功)
- [ ] 2.2 実機で、Vaultがロックされている状態(アンロック画面)でクイックアクセスを開き、Escapeキーでポップアップが閉じることを確認する(issue #76の再現ケース)。
- [ ] 2.3 実機で、検索画面(アンロック済み)でEscapeキーを押すと従来通りポップアップが閉じることを確認する(回帰確認)。
- [ ] 2.4 実機で、ヘルプオーバーレイ表示中にEscapeを押すとヘルプのみが閉じ、ポップアップ自体は表示されたままであることを確認する(回帰確認)。
- [ ] 2.5 実機で、アクションメニュー表示中にEscapeを押すとアクションメニューのみが閉じ、検索画面に戻ることを確認する(回帰確認)。

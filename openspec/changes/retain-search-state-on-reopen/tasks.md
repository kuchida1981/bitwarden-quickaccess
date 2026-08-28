## 1. 非表示時刻の記録

- [x] 1.1 `app/dist/app.js` に非表示時刻を保持する変数(例: `let hiddenAt = null;`)を追加する
- [x] 1.2 Escapeキー押下時・アクション成功時(`runAction`内)など、`invoke("hide_popup")`を呼ぶ箇所すべてで `hiddenAt = Date.now()` を記録する
- [x] 1.3 `window` の `blur` イベントをリッスンし、フォーカスロスによる非表示(`popup.rs`の`WindowEvent::Focused(false)`経由)の場合も同様に `hiddenAt = Date.now()` を記録する

## 2. TTL判定と検索状態保持ロジック

- [x] 2.1 非表示からの経過時間が30秒以内かどうかを判定するヘルパー関数(例: `shouldRetainSearchState()`)を追加する
- [x] 2.2 `syncScreenWithBackend` 内の `actualScreen === lastKnownScreen && actualScreen === "search"` の分岐で、無条件の `searchBox.value = ""` + `runSearch("")` を、TTL判定に基づく条件分岐に置き換える(TTL以内なら何もしない、TTL超過なら従来通りクリアする)
- [x] 2.3 `handleShown` 内の `lastKnownScreen === "search"` の分岐で、TTL以内の場合は `searchBox.focus()` の後に `searchBox.select()` を呼び全選択状態にする
- [x] 2.4 ⌘Lロック(`performLock`)後やバックエンド切断(エラー画面遷移)後は、`actualScreen !== lastKnownScreen` により従来通り検索状態がリセットされることをコードレベルで確認する(新規実装は不要)

## 3. 動作確認

- [ ] 3.1 開発ビルドで、検索して閉じてから30秒以内に再度開くと検索文字列・結果一覧が保持され、テキストが全選択状態になることを確認する
- [ ] 3.2 検索して閉じてから30秒を超えて再度開くと検索状態がクリアされることを確認する
- [ ] 3.3 検索した状態で⌘Lロックし、30秒以内にアンロックしても検索状態が保持されない(空になる)ことを確認する
- [ ] 3.4 検索した状態でバックエンド切断(例: `bw serve`プロセスを一時的に落とす)からの復旧後も検索状態が保持されない(空になる)ことを確認する

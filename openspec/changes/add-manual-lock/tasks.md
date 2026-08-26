## 1. バックエンド: lockコマンドの追加

- [x] 1.1 `app/src-tauri/src/commands.rs` に `lock` コマンドを追加する。既存の `unlock` コマンドと同じパターン(`client_for(&state)?` → `client.lock().await.map_err(...)?` → `state.set_locked()`)で実装する。アイドルタイマーのリセットは行わない。
- [x] 1.2 `app/src-tauri/src/main.rs` の `invoke_handler!` リストに `commands::lock` を追加する。

## 2. トレイメニュー: 「今すぐロック」項目の追加

- [x] 2.1 `app/src-tauri/src/i18n.rs` の `Messages` 構造体に `lock_now_label` フィールドを追加し、日英の文言(例: 日本語「今すぐロック」、英語「Lock Now」)を設定する。
- [x] 2.2 `app/src-tauri/src/tray.rs` に `LOCK_ITEM_ID` 定数と `lock_item`(`MenuItem::with_id`)を追加し、既存のメニュー項目(`status_item`, `hotkey_item` 等)と同様に `Menu::with_items` に含める。初期状態の有効/無効は `initial == BackendState::Unlocked` に基づいて設定する。挿入位置は `autostart_item` と区切り線の間など、状態表示系の項目の近くが自然。
- [x] 2.3 `on_menu_event` のmatch文に `LOCK_ITEM_ID` のケースを追加する。`app.clone()` した `AppHandle` を用いて `tauri::async_runtime::spawn` 内で `crate::commands::lock(app_handle.state::<AppState>()).await` を呼び出す(戻り値のエラーは `eprintln!` で記録する程度でよい、既存の自動起動トグル失敗時の扱いに合わせる)。
- [x] 2.4 既存の状態購読ループ(`status_item`/アイコン更新箇所)に、`lock_item.set_enabled(new_state == BackendState::Unlocked)` の呼び出しを追加する。

## 3. フロントエンド: ⌘Lショートカット

- [x] 3.1 `app/dist/app.js` の `searchBox` のkeydownリスナー内、`isHelpToggleShortcut` の判定の直後(`actionMenuOpen` の判定より前)に、`event.metaKey && event.code === "KeyL"` の判定を追加する。該当する場合 `event.preventDefault()` した上でロック処理(3.2で実装する関数)を呼び、`return` する。
- [x] 3.2 ロック実行用の関数(例: `async function performLock()`)を追加する。`invoke("lock")` を呼び、成功時は `showScreen("unlock")` に切り替え、`lastKnownScreen = "unlock"` を設定し、`passwordInput.value` と `unlockError.textContent` をクリアして `passwordInput.focus()` する。失敗時は何もしない(バックエンド未接続等でロック自体が意味を持たない状況のため)。
- [x] 3.3 `app/dist/index.html` のヘルプオーバーレイに `⌘L` の項目を追加する(`data-i18n` キーは新規に追加し、`app/dist/i18n.js` にも日英エントリを追加する)。

## 4. 動作確認(1〜3の範囲)

- [x] 4.1 `cargo test` を実行し、既存テストが通ることを確認する。(2026-08-26 実行、全35テスト成功、clippy警告なし)
- [x] 4.2 実機で、アンロック済みの検索画面から `⌘L` を押し、Vaultがロックされてアンロック画面に切り替わることを確認する。(2026-08-26 ユーザーによる実機確認完了)
- [x] 4.3 実機で、ロック中・未接続の状態でトレイメニューを開いたとき「今すぐロック」項目が無効化されていることを確認する。(2026-08-26 ユーザーによる実機確認完了)
- [x] 4.4 実機で、⌘Lでロック後、再度マスターパスワードでアンロックできることを確認する(回帰確認)。(2026-08-26 ユーザーによる実機確認完了)

## 5. ポップアップ表示中の状態変化通知(実機確認で発覚した既存の穴への対応)

- [ ] 5.1 `app/src-tauri/src/popup.rs` に `pub const BACKEND_STATE_CHANGED_EVENT: &str = "backend-state-changed";` を追加する(`POPUP_SHOWN_EVENT` の定義の近くが自然)。
- [ ] 5.2 `app/src-tauri/src/main.rs` に、`AppState::subscribe()` を購読し、ポップアップウィンドウが可視状態(`window.is_visible()`)のときだけ `BACKEND_STATE_CHANGED_EVENT` をemitする新しい非同期タスクを追加する(`tray.rs` の状態購読ループと同様のパターン)。`setup()` 内、既存の `start_backend`/`wait_for_shutdown_signal`/`watch_idle_timeout` のspawnと並べて追加する。
- [ ] 5.3 `app/dist/app.js` の `handleShown()` から、`lockState` の取得〜画面切り替えのロジックを `async function syncScreenWithBackend()` として切り出す。切り出した関数の中で、`actualScreen !== "search"` になる場合は `actionMenuOpen`/`actionMenuActions`/`actionMenuFocusIndex`/`helpOpen`/`helpOverlay` のクローズ処理も行う。`handleShown()` は表示時固有の処理(フォーカス初期化等)を行った後に `await syncScreenWithBackend()` を呼ぶ形にする。
- [ ] 5.4 `app/dist/app.js` に `listen("backend-state-changed", () => { syncScreenWithBackend(); });` を追加する(既存の `listen("popup-shown", ...)` の近くが自然)。

## 6. 動作確認(5の範囲)

- [ ] 6.1 `cargo test` を実行し、既存テストが通ることを確認する。
- [ ] 6.2 実機で、検索画面を開いたままトレイメニューから「今すぐロック」を選択し、ポップアップを閉じることなく即座にアンロック画面へ切り替わることを確認する(issue再現ケース)。
- [ ] 6.3 実機で、6.2の後、ポップアップを一度閉じて再度開いても正常にアンロック画面が表示されることを確認する(回帰確認)。
- [ ] 6.4 実機で、通常のフロー(ポップアップを閉じた状態でトレイからロック→ホットキーで開く)が引き続き正常に動作することを確認する(回帰確認)。

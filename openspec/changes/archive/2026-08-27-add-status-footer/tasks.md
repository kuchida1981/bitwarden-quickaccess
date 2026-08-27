## 1. バックエンド実装: ログイン中アカウントの取得

- [x] 1.1 `app/src-tauri/src/backend/http_client.rs` の `StatusTemplate` に `#[serde(rename = "userEmail")] user_email: Option<String>` を追加してパースする。(agy, commit 6506e6c)
- [x] 1.2 `pub struct StatusInfo { pub lock_status: LockStatus, pub user_email: Option<String> }` を追加し、`BwServeClient::status()` の戻り値を `Result<LockStatus, ClientError>` から `Result<StatusInfo, ClientError>` に変更する。(agy, commit 6506e6c)
- [x] 1.3 `app/src-tauri/src/backend/state.rs` の `Inner` に `user_email: Option<String>` を追加し、`AppState` に `set_user_email(&self, email: Option<String>)` / `user_email(&self) -> Option<String>` を、既存の `last_error`/`port` と同じパターンで追加する。(agy, commit 6506e6c)
- [x] 1.4 `app/src-tauri/src/main.rs` の `sync_initial_status` を `status()` の新しい戻り値(`StatusInfo`)に合わせて更新し、Locked/Unlocked判定時に `state.set_user_email(status.user_email)` を呼ぶ。(agy, commit 6506e6c)
- [x] 1.5 `app/src-tauri/src/commands.rs` に `get_current_user(state: tauri::State<'_, AppState>) -> Option<String>` コマンドを追加し、`state.user_email()` を返す。`app/src-tauri/src/main.rs` の `invoke_handler!` リストに追加する。(agy, commit 6506e6c)
- [x] 1.6 `http_client.rs` の既存テスト(`status_parses_locked` 等)を `StatusInfo` に合わせて更新し、`user_email` のパースを検証するテストケースを追加する。`state.rs` に `set_user_email`/`user_email` のテストを追加する。(agy, commit 6506e6c、2026-08-27 `cargo test`/`cargo clippy` 再検証済み)

## 2. ウィンドウ幅の拡大

- [x] 2.1 `app/src-tauri/src/popup.rs` の `WIDTH` 定数を `420.0` から `520.0` に変更する。(agy, commit 5b08807。ウィンドウ位置計算のテストも幅変更に合わせて更新)

## 3. フロントエンド実装: フッタのマークアップ・スタイル

- [x] 3.1 `app/dist/index.html` の `#search-screen` 内、`#empty-message` の後に `<div id="status-footer"><span id="current-user-avatar" class="user-avatar" title=""></span><span id="footer-hints"></span></div>` を追加する。(agy, commit 5b08807)
- [x] 3.2 `app/dist/style.css` に `#status-footer`(横並びflexbox、`justify-content: space-between`、上部ボーダー、パディング)、`.user-avatar`(小さな円形、背景色、中央揃えテキスト、`display: none` を初期値にしJSで表示切替)のスタイルを追加する。既存の `#results li .hints` / `#results li.focused .hints` のスタイルは、行内ヒント表示を削除するタスク(4系)に合わせて削除する。(agy, commit 5b08807)

## 4. フロントエンド実装: 行内ヒントの削除とフッタへの反映

- [x] 4.1 `app/dist/app.js` の `buildTrailingBlock` を変更し、非アクションメニュー分岐(現状 `.hints` divを生成している部分)を、空の `<span class="row-trailing-placeholder"></span>` を返すように変更する。(agy, commit 5b08807)
- [x] 4.2 `app/dist/app.js` に、DOM参照 `statusFooter` / `currentUserAvatar` / `footerHints`(`document.getElementById` で取得)を追加する。(agy, commit 5b08807)
- [x] 4.3 `initI18n().then(...)` 内(`SHORTCUT_HINTS = t("shortcutHints")` の箇所)で、`footerHints.textContent = SHORTCUT_HINTS` も設定する。(agy, commit 5b08807)
- [x] 4.4 検索画面へ遷移するタイミング(`syncScreenWithBackend`/`handleShown` で `actualScreen === "search"` になる箇所)で `invoke("get_current_user")` を呼び出し、結果に応じて `currentUserAvatar` の `textContent`(先頭1文字・大文字化)と `title`(完全なメールアドレス)を設定し、`None` の場合は `currentUserAvatar.style.display = "none"` にする(取得できた場合は表示に戻す)。(agy, commit 5b08807、`refreshCurrentUser` として実装)

## 5. 動作確認

- [x] 5.1 `cargo test` を実行し、既存テストおよび追加したテストが通ることを確認する。(2026-08-27 全38テスト成功)
- [x] 5.2 `cargo clippy --all-targets -- -D warnings` を実行し、警告が無いことを確認する。(2026-08-27 警告なし)
- [x] 5.3 `node --check app/dist/app.js` で構文エラーが無いことを確認する。(2026-08-27 確認済み)

## 6. セキュリティレビュー

- [x] 6.1 `/security-review` を実行する。(2026-08-27 実行、confidence 8/10以上の指摘なし。`get_current_user` は既存の `get_backend_error` と同一パターンでの露出であり、フロントエンド側も `textContent`/`title` によるプロパティ代入のみでXSSリスクなしと確認)
- [x] 6.2 `/code-review` を実行する。(2026-08-27 実行、8つの観点で精査、指摘なし)

## 7. 実機確認

- [x] 7.1 (ユーザー作業)実機で、検索画面下部にフッタが表示され、ショートカットヒントが行内ではなくフッタに表示されることを確認する。(2026-08-27 確認完了、問題なし)
- [x] 7.2 (ユーザー作業)実機で、フッタのアカウントアバターにカーソルを合わせると、ログイン中アカウントの正しいメールアドレスがtooltip表示されることを確認する。(2026-08-27 確認完了、問題なし)
- [x] 7.3 (ユーザー作業)実機で、広げたウィンドウ幅でフッタ(アバター+ヒント)が1行に収まって表示されることを確認する。収まらない場合は `popup.rs` の `WIDTH` を調整する。(2026-08-27 確認完了、問題なし)
- [x] 7.4 (ユーザー作業)実機で、アクションメニューの開閉・行フォーカス移動(`add-item-icons` で修正した挙動)に回帰が無いことを確認する。(2026-08-27 確認完了、問題なし)

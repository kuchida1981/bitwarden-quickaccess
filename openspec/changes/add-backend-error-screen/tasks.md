## 1. バックエンド: エラーメッセージ取得コマンドの追加

- [ ] 1.1 `app/src-tauri/src/commands.rs` に `get_backend_error` コマンドを追加する。シグネチャは `fn get_backend_error(state: tauri::State<'_, AppState>) -> Option<String>` とし、内部で `state.last_error()` を返す。既存の `get_lock_state` と同様に `#[tauri::command]` を付与する。
- [ ] 1.2 `app/src-tauri/src/main.rs` の `invoke_handler!` リストに `commands::get_backend_error` を追加する。

## 2. フロントエンド: エラー画面の追加

- [ ] 2.1 `app/dist/index.html` に `<section id="error-screen" class="screen">` を追加する。見出し(`data-i18n="errorScreenTitle"`)とエラーメッセージ表示用の要素(例: `<p id="error-message"></p>`)を含める。既存の `unlock-screen`/`search-screen` と同じ `.screen` クラスの構造に合わせる。
- [ ] 2.2 `app/dist/app.js` に `errorScreen`(`document.getElementById("error-screen")`)と `errorMessage`(`document.getElementById("error-message")`)のDOM参照を追加する。
- [ ] 2.3 `showScreen(name)` 関数に `errorScreen.classList.toggle("active", name === "error")` を追加する。
- [ ] 2.4 `handleShown()` 内の画面判定ロジックを、`lockState` の3値に対応させる: `"unlocked"` → `"search"`、`"locked"` → `"unlock"`、それ以外(`"disconnected"`)→ `"error"`。`actualScreen === "error"` の場合は `invoke("get_backend_error")` を呼び、その結果を `errorMessage.textContent` にセットする(取得失敗時は空のままでよい)。
- [ ] 2.5 `app/dist/i18n.js` に `errorScreenTitle` の日英エントリを追加する(例: 日本語「接続できません」、英語「Connection Error」)。

## 3. 動作確認

- [ ] 3.1 `cargo test` を実行し、既存テストが通ることを確認する。
- [ ] 3.2 実機で、`bw` コマンドをPATHから一時的に外すかリネームした状態でアプリを起動し、クイックアクセスを開いた際にエラー画面(具体的なエラーメッセージ付き)が表示され、マスターパスワード入力フォームが表示されないことを確認する。
- [ ] 3.3 実機で、通常の(`bw` が正しくインストールされ、ログイン済みの)環境でロック中・アンロック済みそれぞれの画面が従来通り表示されることを確認する(回帰確認)。

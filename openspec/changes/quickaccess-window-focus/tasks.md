## 1. カーソル位置ベースのディスプレイ表示(#55)

- [ ] 1.1 `app/src-tauri/src/popup.rs` に、`app: &AppHandle` を受け取り「表示すべき位置(x, y)」を計算する関数(例: `fn compute_popup_position(app: &AppHandle) -> (f64, f64)`)を新設する。ロジック: `app.cursor_position()` → 成功したら `app.monitor_from_point(x, y)` でディスプレイを特定 → 取得できたディスプレイの中央上部の座標を返す。いずれかの呼び出しが失敗/`None`の場合は既存の `app.primary_monitor()` の結果にフォールバックする(現状の `create_popup_window` 内のロジックを移植・流用する)
- [ ] 1.2 `create_popup_window` から起動時の位置決め処理(`primary_monitor()` を使った `builder.position(x, y)` 呼び出し)を削除する(ウィンドウ生成自体は残す。初期位置は仮の値でよい。すぐ `toggle_popup` 初回呼び出しで上書きされる)
- [ ] 1.3 `toggle_popup` の表示(show)分岐で、`window.show()` を呼ぶ前に 1.1 の関数で位置を計算し、`window.set_position(...)` で反映する
- [ ] 1.4 単体テスト: 1.1 の位置計算ロジックのうち、モニター座標から中央上部座標を算出する部分(既存の `(monitor_size.width - WIDTH) / 2.0` 等の計算式)を関数として切り出し、既知のモニターサイズ入力に対する期待座標を検証するテストを追加する(`app.cursor_position()` 等の実OS呼び出し部分はテスト対象外でよい)

## 2. 直前アプリへのフォーカス復帰(#56)

- [ ] 2.1 `app/src-tauri/Cargo.toml` に `objc2-app-kit`(バージョンはCargo.lockで既に解決されている0.3系に合わせる。例: `objc2-app-kit = "0.3"`)を追加する。`objc2` が直接必要になる場合は同様に追加する
- [ ] 2.2 `app/src-tauri/src/popup.rs` に `struct PreviousFrontmostApp(std::sync::Mutex<Option<libc::pid_t>>)` を定義する
- [ ] 2.3 `main.rs` の `tauri::Builder` チェーンに `.manage(PreviousFrontmostApp(std::sync::Mutex::new(None)))` を追加する(既存の `.manage(lang)` 等と同じ場所)
- [ ] 2.4 `toggle_popup` の表示(show)分岐で、`window.show()` を呼ぶ**前**に `objc2_app_kit::NSWorkspace::sharedWorkspace().frontmostApplication()` を呼び、取得できた場合はその `processIdentifier()` を `PreviousFrontmostApp` に保存する(取得できなければ `None` のままにする)
- [ ] 2.5 `toggle_popup` の非表示(hide)分岐と、`commands::hide_popup` コマンド(`app/src-tauri/src/commands.rs`。コピー操作後にフロントエンドから呼ばれる)の両方から呼べる共通関数(例: `fn restore_previous_focus(app: &AppHandle)`)を実装する。処理内容: `PreviousFrontmostApp` からPIDを `take()` し、`Some(pid)` であれば `NSRunningApplication::runningApplicationWithProcessIdentifier(pid)` を呼び、取得できれば `activateWithOptions(NSApplicationActivationOptions::empty())` を呼ぶ。`window.hide()` の**後**に呼び出す
- [ ] 2.6 `toggle_popup` の非表示分岐、および `commands::hide_popup` の両方で 2.5 の関数を呼び出すよう配線する(design.md 決定3 参照。二重に呼ばれても副作用がない設計にする — `take()` により2回目は `None` になるため自然に冪等になる)

## 3. 動作確認・仕上げ

- [ ] 3.1 `cd app/src-tauri && cargo build && cargo clippy --all-targets -- -D warnings && cargo test` が通ることを確認する
- [ ] 3.2 マルチディスプレイ環境(または外部ディスプレイ接続時)で、カーソルを外部ディスプレイに置いた状態でホットキーを押し、そのディスプレイにポップアップが表示されることを確認する(実機確認が必要)
- [ ] 3.3 シングルディスプレイ環境で、従来と同じ位置(画面中央上部)に表示されることを確認する(実機確認が必要)
- [ ] 3.4 他アプリ(ブラウザ等)を操作中にホットキーでポップアップを表示し、ユーザー名/パスワード/TOTPのいずれかをコピーした後、元のアプリにフォーカスが戻り、そのままペースト操作できることを確認する(実機確認が必要)
- [ ] 3.5 他アプリを操作中にホットキーでポップアップを表示し、何もせず再度ホットキーで閉じた場合も元のアプリにフォーカスが戻ることを確認する(実機確認が必要)
- [ ] 3.6 `specs/quickaccess-window-focus/spec.md` の各シナリオが満たされていることを確認する

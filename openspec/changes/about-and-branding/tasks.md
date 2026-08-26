## 1. アイコンアセットの生成・差し替え(#50)

- [x] 1.1 承認済みのSVGソース(アプリ本体アイコン、トレイシールド)を `app/src-tauri/icons/source/app-icon.svg` と `app/src-tauri/icons/source/tray-shield.svg`(`fill` を状態色に差し替えるテンプレート)としてリポジトリに追加する
- [x] 1.2 `app/src-tauri/icons/source/app-icon.svg` から `rsvg-convert` で以下を生成し、既存ファイルを差し替える: `icon.png`(128x128)、`128x128.png`、`128x128@2x.png`(256x256)、`32x32.png`
- [x] 1.3 `icon.icns` を作り直す: `rsvg-convert` で `icon_16x16.png`/`icon_16x16@2x.png`/`icon_32x32.png`/`icon_32x32@2x.png`/`icon_128x128.png`/`icon_128x128@2x.png`/`icon_256x256.png`/`icon_256x256@2x.png`/`icon_512x512.png`/`icon_512x512@2x.png` を生成して `.iconset` ディレクトリにまとめ、`iconutil -c icns` で `icon.icns` を生成する(初回、iconsetディレクトリ名の末尾が `.iconset` になっておらず `iconutil` が「Invalid Iconset」で失敗し古いファイルが残っていたことに気づき、正しい命名で再実行して解消)
- [x] 1.4 `app/src-tauri/icons/source/tray-shield.svg` の `fill` を各状態色(`#8C8C8C` / `#D97706` / `#16A34A`)に差し替えたSVGから、`tray-disconnected.png` / `tray-locked.png` / `tray-unlocked.png`(いずれも32x32)を生成し、既存ファイルを差し替える
- [x] 1.5 `cd app/src-tauri && cargo build` が通ることを確認する(`include_bytes!` でアイコンファイルを埋め込んでいるため、ファイル差し替え後もビルドできることの確認)

## 2. トレイメニューへのアプリ名・バージョン・リポジトリリンク追加(#57)

- [x] 2.1 `app/src-tauri/src/i18n.rs` の `Messages` 構造体に `repo_link_label: &'static str` フィールドを追加し、`JA`(「GitHubリポジトリを開く」)・`EN`("View on GitHub")それぞれに値を設定する(既存の `version_label` フィールドは用途がなくなったため削除)
- [x] 2.2 `app/src-tauri/src/tray.rs` の `VERSION_ITEM_ID` / `version_item` を、`ABOUT_ITEM_ID` の非活性項目(ラベルは `format!("{name} v{version}", name = app.package_info().name, version = APP_VERSION)`)に置き換える
- [x] 2.3 `tray.rs` に新規定数 `REPO_LINK_ITEM_ID` と、クリック可能な `repo_link_item`(ラベルは `m.repo_link_label`)を追加し、メニュー項目リストに `about_item` の直後・`quit_item` の直前として組み込む
- [x] 2.4 `tray.rs` の `on_menu_event` の `match` に `REPO_LINK_ITEM_ID` の分岐を追加し、`tauri_plugin_opener::OpenerExt` の `app.opener().open_url("https://github.com/kuchida1981/bitwarden-quickaccess", None::<&str>)` を呼ぶ(`commands.rs` の `open_in_browser` と同じ使い方)。`tray.rs` の先頭に `use tauri_plugin_opener::OpenerExt;` を追加する(agyへの委譲がタイムアウトしたため、この分岐のみClaude Codeが直接追加)
- [x] 2.5 `cd app/src-tauri && cargo build && cargo clippy --all-targets -- -D warnings && cargo test` が通ることを確認する

## 3. 動作確認・仕上げ

- [ ] 3.1 実機でトレイメニューを開き、シールド形状のアイコンと、アプリ名+バージョンの表示、GitHubリポジトリを開くリンクが表示されていることを確認する(実機確認が必要)
- [ ] 3.2 実機でリポジトリリンクをクリックし、既定のブラウザで本リポジトリのページが開くことを確認する(実機確認が必要)
- [ ] 3.3 実機でロック中/アンロック済み/未接続それぞれの状態でトレイアイコンの色とシールド形状が正しく切り替わることを確認する(実機確認が必要)
- [ ] 3.4 アプリのDockアイコン相当(`.app`をFinderで表示した際のアイコン)が新しいデザインになっていることを確認する(実機確認が必要。`cargo tauri build` でのバンドルが必要な場合がある点に注意)
- [ ] 3.5 `specs/about-and-branding/spec.md` の各シナリオが満たされていることを確認する

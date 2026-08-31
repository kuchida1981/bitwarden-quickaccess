## 1. Bundle identifier の変更

- [ ] 1.1 `app/src-tauri/tauri.conf.json` の `identifier` を `com.kuchida1981.bw-quickaccess` から `com.u-rei.bw-quickaccess` に変更し、`git diff` で他の値(`productName` 等)が変わっていないことを確認する

## 2. 検証

- [ ] 2.1 `cargo build` / `cargo test` / `cargo clippy --all-targets -- -D warnings`(`app/src-tauri` 配下)を実行し、いずれも成功することを確認する
- [ ] 2.2 `cargo tauri build`(またはセルフビルド手順)でアプリをビルドし、生成された `.app` の `Info.plist` の `CFBundleIdentifier` が `com.u-rei.bw-quickaccess` になっていることを確認する
- [ ] 2.3 開発者本人の環境で新しいビルドに更新し、グローバルホットキー(Shift+Cmd+Space)の動作を確認する。動作しない場合はSystem Settings > Privacy & Security > Accessibilityで新しいアプリのエントリを許可し、再度動作を確認する

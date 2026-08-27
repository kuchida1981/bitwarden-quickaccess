## 1. アプリ設定の変更

- [x] 1.1 `app/src-tauri/tauri.conf.json` の `productName` を `"bw-quickaccess"` から `"Bitwarden Quick Access"` に変更する(`identifier` は変更しない)
- [x] 1.2 ローカルでビルドし(`cargo tauri build` 相当)、生成される `.app` バンドル名が `Bitwarden Quick Access.app` になることを確認する

## 2. ドキュメント更新

- [x] 2.1 `README.md` 内の `bw-quickaccess.app` / `bw-quickaccess_aarch64.app.tar.gz` の記載を新しい名称ベースに更新する
- [x] 2.2 `README.ja.md` 内の同様の記載を新しい名称ベースに更新する
- [x] 2.3 `CONTRIBUTING.md` のリリース手順に、Homebrew tap の `Casks/bw-quickaccess.rb`(`app`/`url`/`caveats`)を手動更新する旨の注記(`url` 行のスペースを `%20` でパーセントエンコードする方法を含む)を追加する

## 3. 検証

- [x] 3.1 `cargo test` を実行し全テストが成功することを確認する
- [x] 3.2 `cargo clippy --all-targets -- -D warnings` を実行し警告がないことを確認する
- [x] 3.3 実機確認: ビルドしたアプリを起動し、Finder上のアプリ名・トレイメニューの表示・グローバルホットキー(アクセシビリティ権限の再許可が不要なこと)を確認する
- [x] 3.4 実機確認: 既存の自動起動設定(有効化している場合)がアップデート後にどう表示されるかを確認し、必要であれば手動でオン/オフを切り替え直す
  - Homebrewでインストール済みだった旧ビルド(`bw-quickaccess`)で自動起動がオンになっていることを確認し、オフに切り替え済み。`tauri-plugin-autostart`のオフ処理は該当LaunchAgent plistファイル自体を削除するため、旧`bw-quickaccess.plist`は残存しない。

## 4. リリース時対応(次回リリース実施時)

- [ ] 4.1 次回リリース公開後、tapリポジトリ(`kuchida1981/homebrew-bitwarden-quickaccess`)の `Casks/bw-quickaccess.rb` の `app`/`url`/`caveats` を新しいアセット名(スペースを `%20` エンコードした `url`)に手動で更新する
- [ ] 4.2 `brew style --cask bw-quickaccess` / `brew audit --cask bw-quickaccess` / `brew reinstall --cask bw-quickaccess` で確認する

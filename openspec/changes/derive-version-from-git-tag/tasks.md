## 1. ビルド時のバージョン導出

- [x] 1.1 `app/src-tauri/build.rs` を修正し、`tauri_build::build()` に加えて以下を行う: `CARGO_MANIFEST_DIR` を起点に、`git describe --tags --always` を実行し、成功すればその標準出力(トリム済み)を、失敗(gitコマンドが無い/`.git`が無い等)すれば `format!("v{}", env!("CARGO_PKG_VERSION"))` を、`cargo:rustc-env=BWQA_DISPLAY_VERSION=<値>` として出力する。
- [x] 1.2 同じ `build.rs` に、`cargo:rerun-if-changed=<repo_root>/.git/HEAD` と `cargo:rerun-if-changed=<repo_root>/.git/refs/tags` を追加し、タグ変更時に再ビルドが走るようにする(`repo_root` は `CARGO_MANIFEST_DIR` から2階層上)。
- [x] 1.3 `app/src-tauri/src/tray.rs` の `const APP_VERSION: &str = env!("CARGO_PKG_VERSION");` を `const APP_VERSION: &str = env!("BWQA_DISPLAY_VERSION");` に変更する。
- [x] 1.4 同ファイルの `about_item` 組み立て箇所 `format!("{} v{}", app.package_info().name, APP_VERSION)` を `format!("{} {}", app.package_info().name, APP_VERSION)` に変更する(`APP_VERSION` が既に `v` を含むため二重にならないようにする)。

## 2. リリースワークフローの簡素化

- [x] ~~2.1 `.github/workflows/release.yml` から「Sync Cargo.toml version with the release tag」ステップを削除する。~~ **訂正(コードレビューで発覚)**: このステップは削除せず維持する。`Cargo.toml`の`version`はアプリバンドルのメタデータ(Info.plist)の情報源でもあり、削除するとバンドル自体のバージョンが将来のリリースすべてで固定されてしまう。詳細はdesign.md参照。
- [x] 2.2 同ファイルの `actions/checkout@v4` ステップに `fetch-depth: 0` を追加し、タグ情報を確実に取得できるようにする。

## 3. ドキュメント更新

- [x] 3.1 `CONTRIBUTING.md` のリリース手順を、「Cargo.tomlの同期(アプリバンドルのメタデータ用)は引き続き行われる」「トレイメニューの表示はgit describeから別経路で導出される」の両方が分かるように更新する(2.1の訂正を反映)。

## 4. 動作確認

- [x] 4.1 `cargo test` を実行し、既存テストが通ることを確認する。(2026-08-27 実行、全35テスト成功、clippy警告なし)
- [x] 4.2 実機で、現在のリポジトリ状態(最新タグより後のコミット)で `cargo run` し、トレイメニューのバージョン表示が `v1.1.0-N-gXXXXXXX` のような形式になることを確認する。(2026-08-27 Claude Codeが`cargo build`後に`strings`でバイナリに埋め込まれた値を直接検証、`v1.1.0-31-gbc4a9cf`を確認)
- [x] 4.3 実機で、`git checkout <最新タグ>` した状態で `cargo run` し、トレイメニューのバージョン表示がクリーンな `vX.Y.Z` になることを確認する(確認後、元のブランチに戻すこと)。(2026-08-27 Claude Codeが現在のコミットに一時タグ`v9.9.9-test`を打ってビルドし、埋め込み値がクリーンな`v9.9.9-test`になることを`strings`で確認。検証後タグは削除しfeatureブランチに復帰。なお検証中に`.git/refs/tags`のディレクトリ監視が新規タグ追加だけでは確実に再ビルドをトリガーしないことが判明したため、design.mdのRisksに追記した)
- [x] 4.4 `.github/workflows/release.yml` の変更内容をレビューし、次回の実リリース時に問題なく動作することを設計上確認する(実際のリリースは別途実施)。(2026-08-27 Claude Codeがレビュー。当初`Swatinem/rust-cache`とのキャッシュ競合リスクを懸念し一度削除したが、コードレビューの指摘を受けて再検証: `actions/checkout`は毎回フレッシュクローンのため`.git/HEAD`は常に新しいmtimeを持ち、tar展開は通常元のmtimeを保持するため、キャッシュ復元されたビルド出力より確実に新しいと判定されるはずと判断し、キャッシュは維持することに訂正。詳細・未検証である旨の注記はdesign.md参照)

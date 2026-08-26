## 1. 前提確認

- [x] 1.1 `bw-serve-backend` / `menubar-hotkey-shell` / `quickaccess-search-ui` / `credential-actions-autolock` の全タスクが完了し、GUIアプリで検索・コピー・ブラウザ起動・自動ロックの一連の動作が確認済みであることを確認する(全4change、ユーザーによる実機確認済みでマージ完了)

## 2. 旧TUI削除

- [x] 2.1 `bin/bw-quickaccess` を削除する
- [x] 2.2 `lib/*.sh` を削除する(`lib/i18n/*.sh` を含め `lib/` ディレクトリ全体を削除)
- [x] 2.3 `install.sh` を削除する
- [x] 2.4 `test/lib/*.bats` および `test/helpers/*.bash` を削除する(加えて、旧TUI専用でありタスク一覧に明記はなかったが削除後に完全に無用となる `script/build.sh`(TUIのバンドルスクリプト)と `test/fixtures/*.json`(削除したbatsテスト専用のフィクスチャ)も合わせて削除した)

## 3. CI更新

- [x] 3.1 `.github/workflows/ci.yml` から bash構文チェック・shellcheck・bats実行のステップを削除する
- [x] 3.2 `.github/workflows/ci.yml` に macOS runnerでの `cargo build` / `cargo test` / `cargo clippy` ステップを追加する

## 4. リリースワークフロー更新

- [x] 4.1 `.github/workflows/release.yml` から旧TUI向けバンドル・i18nメッセージファイル同梱ロジックを削除する
- [x] 4.2 `.github/workflows/release.yml` に `tauri build` によるmacOS `.app` ビルドとリリースアセット添付を追加する(`tauri-apps/tauri-action` を使用)。実機で `cargo tauri build` を実行し `.app` バンドルが正しく生成されることを確認済み
- [x] 4.3 アプリ内メニューから確認できるバージョン表示(`tauri.conf.json`/`Cargo.toml` のversionを情報源とする)を実装する(`tauri.conf.json` からversionフィールドを削除しCargo.tomlを単一の情報源とし、トレイメニューに `env!("CARGO_PKG_VERSION")` によるバージョン表示項目を追加。実機で `tauri build` 後の `.app` のバージョンが正しく反映されることを確認済み)

## 5. README刷新

- [x] 5.1 `README.md` をGUIアプリ前提の内容(必要要件・セルフビルド手順・Releasesからのインストール手順・使い方=ホットキー/アクション一覧)に全面書き換えする
- [x] 5.2 `README.ja.md` を同様に全面書き換えする
- [x] 5.3 Gatekeeper警告時の回避手順(右クリック→開く)を明記する
- [x] 5.4 既存curlインストール済みユーザー向けの旧TUIアンインストール手順を明記する

## 6. 最終確認

- [ ] 6.1 `openspec/specs/` 配下の該当capability(本changeのspecsで REMOVED としたもの)が正しく反映されることを `/opsx:archive` 実行時に確認する(archive実行時に確認予定)
- [ ] 6.2 クリーンな環境(またはクリーンなgit worktree)で `git clone` + `tauri build` によるセルフビルド手順がREADME記載通りに動作することを確認する(現在の作業ツリー上で `cargo tauri build` の成功と `.app` バンドル生成・バージョン反映は確認済み。完全にクリーンな `git clone` からの検証はユーザー確認待ち)

## 1. 既存コードのフォーマット適用

- [x] 1.1 `app/src-tauri` で `cargo fmt` を実行し、対象の12ファイル(`build.rs`, `src/backend/clipboard_guard.rs`, `src/backend/http_client.rs`, `src/backend/idle.rs`, `src/backend/preflight.rs`, `src/backend/process.rs`, `src/backend/state.rs`, `src/commands.rs`, `src/i18n.rs`, `src/main.rs`, `src/popup.rs`, `src/tray.rs`)を整形する。整形後に `cargo fmt --check` を実行し、差分がゼロであることを確認する
- [x] 1.2 フォーマット適用後に `cargo build --verbose` / `cargo test --verbose` / `cargo clippy --all-targets -- -D warnings` を実行し、いずれも整形前と同じ結果(成功、テスト全通過、警告ゼロ)になることを確認する
- [x] 1.3 フォーマット差分のみを `chore: apply cargo fmt to app/src-tauri` としてコミットする(ロジック変更を含めない)

## 2. CI ワークフローへの fmt チェック追加

- [x] 2.1 `.github/workflows/ci.yml` の `test` ジョブに `cargo fmt --check` ステップを追加する。個別の `working-directory` は指定せず、既存ステップと同様に job レベルの `defaults.run.working-directory: app/src-tauri` に従わせる
- [x] 2.2 追加後の `ci.yml` を読み直し、ステップ名・実行順序(`cargo build` → `cargo test` → `cargo fmt --check` → `cargo clippy`、もしくは同等の妥当な順序)が既存の3ステップと一貫したスタイルになっていることを確認する
- [x] 2.3 CI ステップ追加のみを `ci: add cargo fmt --check step` としてコミットする(フォーマット差分とは別コミットにする)

## 3. 検証

- [x] 3.1 `app/src-tauri` で `cargo fmt --check` をローカル実行し、追加した CI ステップと同じコマンドが差分なしで成功することを確認する
- [x] 3.2 PR 作成後に `gh pr checks --watch` で CI(`cargo build` / `cargo test` / `cargo fmt --check` / `cargo clippy`)がすべて成功することを確認する(PR #138, 1m2sでpass)

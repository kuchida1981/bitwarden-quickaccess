## 1. bwパス解決モジュールの新規実装

- [ ] 1.1 `app/src-tauri/src/backend/bw_locate.rs`(新規モジュール)を作成し、`app/src-tauri/src/backend/mod.rs` に登録する。設定ファイルパス(`$XDG_CONFIG_HOME/bw-quickaccess/bw_path.txt`、未設定時 `~/.config/bw-quickaccess/bw_path.txt`)・既知パスのリスト(`/opt/homebrew/bin/bw` → `/usr/local/bin/bw`)・PATHフォールバック(`"bw"`)を順に試す解決関数(例: `resolve_bw_path() -> String`)を実装する。既存の `preflight::check_bw_cli_with` と同様に、候補パス群を引数として注入できる内部関数(例: `resolve_bw_path_with(config_path: Option<&Path>, known_paths: &[&str]) -> String`)に切り出し、`resolve_bw_path()` はそれを固定引数で呼ぶ薄いラッパーとする。`cargo test` で当該モジュールのユニットテストが通ることを確認する。
- [ ] 1.2 設定ファイルが (a) 存在しない、(b) 存在するが記載パスが実行可能ファイルとして存在しない、(c) 存在し記載パスが有効、の3パターンをユニットテストでカバーする(一時ディレクトリに実ファイルを配置する方式。`preflight.rs` の既存テストパターンを踏襲)。`cargo test` で通ることを確認する。
- [ ] 1.3 既知パス探索とPATHフォールバックの優先順位(設定ファイル → 既知パス → `"bw"`)をユニットテストで検証する(既知パスの実在確認は一時ディレクトリに配置したダミー実行ファイルで代替し、実際の `/opt/homebrew` 等には依存しない形にする)。`cargo test` で通ることを確認する。

## 2. 既存呼び出し箇所の書き換え

- [ ] 2.1 `app/src-tauri/src/backend/preflight.rs` の `PreflightError::BwNotFound` エラーメッセージを更新し、設定ファイルの場所(`$XDG_CONFIG_HOME/bw-quickaccess/bw_path.txt` または `~/.config/bw-quickaccess/bw_path.txt`)と書き方を案内する文言を含める。既存テスト(`reports_bw_not_found_when_command_missing` 等)を新しいメッセージに合わせて更新し、`cargo test` で通ることを確認する。
- [ ] 2.2 `app/src-tauri/src/backend/process.rs` の `build_bw_serve_command(port: u16)` を `build_bw_serve_command(port: u16, bw_path: &str)` に変更し、`Command::new("bw")` を `Command::new(bw_path)` に置き換える。既存テスト `build_bw_serve_command_binds_to_ipv4_loopback` を新シグネチャに合わせて更新し、`cargo test` で通ることを確認する。
- [ ] 2.3 `app/src-tauri/src/main.rs` の `start_backend()` 内で `bw_locate::resolve_bw_path()` を呼び出し、その結果を `preflight::check_bw_cli_with()` と `process::build_bw_serve_command()`(呼び出し元クロージャ)の両方に渡すよう変更する。`cargo build` が通ることを確認する。
- [ ] 2.4 `app/src-tauri/src/main.rs` から `fix_path_env()` / `PATH_MARKER` / `extract_path_from_marker` / `run_shell_and_capture_stdout` および `main()` 冒頭の `fix_path_env();` 呼び出し、`mod fix_path_env_tests` を削除する。`cargo build` と `cargo clippy --all-targets -- -D warnings` が警告なく通ることを確認する。

## 3. ドキュメント更新

- [ ] 3.1 `README.md` に、Homebrew以外の方法(nvm等のnode管理ツール経由のnpmインストール、ネイティブバイナリの手動配置)で `bw` をインストールした場合の設定ファイル手順(パス・書式の具体例)を追記する。
- [ ] 3.2 `CONTRIBUTING.md` にこの変更に関連する開発者向け留意事項(該当箇所があれば)を確認し、必要な場合のみ追記する。

## 4. 動作確認

- [ ] 4.1 macOS実機(Apple Silicon)で以下を確認する: (a) Homebrewでインストールした `bw` が既知パス経由で認識され通常起動できる、(b) `bw` を既知パス外の場所に置き設定ファイルでそのパスを指定した状態で起動できる、(c) 設定ファイル・既知パスのいずれも無効な状態で「バックエンド未接続」エラーと、設定ファイルへの案内メッセージが表示される。`cargo build` / `cargo test` / `cargo clippy --all-targets -- -D warnings` が全て通ることも合わせて確認する。

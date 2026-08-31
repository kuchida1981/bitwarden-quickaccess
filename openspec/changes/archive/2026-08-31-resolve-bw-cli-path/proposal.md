## Why

`fix_path_env()`(`app/src-tauri/src/main.rs`)は、Finder起動やログイン項目からの自動起動でPATHが最小限になり`bw`コマンドが見つからない問題を回避するため、ユーザーのログインシェルを実際にspawnして`printenv PATH`を取得し、プロセス全体のPATHを`unsafe { std::env::set_var(...) }`で書き換えている。この設計はシェル起動ファイルの中身に強く依存しており、起動の遅延・タイムアウト(Issue #83で対処済みだがそもそもの複雑さの根)、`exec fish`のようなシェル間ブートストラップ構成でのPATH欠落(Issue #83実機確認で発生)といった脆さを抱える。加えて、プロセス全体の環境変数を書き換えるこの手法自体がRustの`set_var`スレッド安全性まわりの`unsafe`を要求している。

Bitwarden CLI(`bw`)の主要なインストール経路(Homebrew formula、Homebrewのnodeでの`npm install -g`)は`brew --prefix`配下([Apple Silicon: `/opt/homebrew/bin`, Intel/Rosetta: `/usr/local/bin`])に収束することが確認できたため、シェルを起動せずファイルシステムを直接探索する方式に置き換えられる。既知パスでカバーしきれないケース(nvm等のバージョン管理node経由のnpmインストール、ネイティブバイナリの手動配置)は、ユーザーが明示的にパスを指定できる設定ファイルで救済する。

## What Changes

- `fix_path_env()` / `PATH_MARKER` / `run_shell_and_capture_stdout` とその関連テストを削除し、ログインシェルをspawnする方式を完全に撤廃する
- `bw`実行ファイルの解決を新設: 以下の順で最初に見つかったものを採用する
  1. 設定ファイル(`$XDG_CONFIG_HOME/bw-quickaccess/bw_path.txt`、未設定時は`~/.config/bw-quickaccess/bw_path.txt`)に書かれた絶対パス
  2. 既知のインストール先: `/opt/homebrew/bin/bw` → `/usr/local/bin/bw`
  3. 素の`"bw"`(現在のプロセスが継承しているPATHでの解決。ターミナル起動等の救済用フォールバック)
- 解決処理を`main()`起動前(tokioランタイムなしの同期処理)から、`start_backend()`内(非同期・`app_handle`利用可能)へ移動する
- `unsafe { std::env::set_var("PATH", ...) }` によるプロセス全体のPATH書き換えを撤去し、解決した絶対パス(または`"bw"`)を`preflight::check_bw_cli_with()` / `process::build_bw_serve_command()`へ直接渡す方式に変更する
- `bw`が見つからない場合のエラーメッセージ(`PreflightError::BwNotFound`)を更新し、設定ファイルでの明示指定方法を案内する
- README(場合により CONTRIBUTING.md)に、Homebrew以外の方法(nvm経由のnpmインストール、ネイティブバイナリ手動配置等)で`bw`をインストールした場合の設定ファイル手順を追記する

## Capabilities

### New Capabilities

(なし)

### Modified Capabilities

- `vault-backend-service`: 「bw serve CLI 前提チェック」要件における`bw`実行ファイルの検出方法を、ログインシェル起動によるPATH取得から、設定ファイルオーバーライド → 既知インストール先 → プロセス継承PATHの順次探索に変更する。あわせて、設定ファイルでの明示指定シナリオと、いずれの方法でも見つからない場合のエラー内容(設定ファイルへの案内を含む)をシナリオとして追加する。

## Impact

- `app/src-tauri/src/main.rs`: `fix_path_env()`関連コードの削除、`start_backend()`でのbwパス解決呼び出しの追加
- `app/src-tauri/src/backend/preflight.rs`: `check_bw_cli()`が解決済みパスを受け取るように変更、`BwNotFound`エラーメッセージ更新
- `app/src-tauri/src/backend/process.rs`: `build_bw_serve_command()`が解決済みパスを受け取るように変更
- 新規モジュール(bwパス解決ロジック: 設定ファイル読み込み・既知パス探索): `app/src-tauri/src/backend/`配下に追加予定(design.mdで確定)
- `openspec/specs/vault-backend-service/spec.md`: delta spec追加
- `README.md`(必要に応じて`CONTRIBUTING.md`): 設定ファイルによるbwパス指定手順の追記
- 依存クレートの追加なし(標準ライブラリの`std::fs`・`std::env`のみで実装可能な想定)

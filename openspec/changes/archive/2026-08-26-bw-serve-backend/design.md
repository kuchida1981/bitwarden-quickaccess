## Context

現行の `bin/bw-quickaccess`(bash + fzf)は `bw` コマンドをキー入力のたびにサブプロセスとして都度起動する。1Password Quick Access相当の常駐GUIアプリ(後続change群)では、ホットキー押下からポップアップ表示までを高速に行う必要があり、この使い捨てサブプロセスパターンは踏襲しない。

`bw` CLI は `bw serve` サブコマンドを持ち、vaultをアンロックした状態でメモリに保持したまま `localhost` にREST APIを立てられることを実機で確認済み(`bw serve --hostname localhost --port <port>` → `/status` はロック状態を返し、`/list/object/items` はロック中に `{"success": false, "message": "Vault is locked."}` を返す、`/generate` はロック不要、`/lock` は冪等)。認証・vault復号・TOTP計算はBitwarden公式実装にそのまま委譲でき、自前実装のリスク(暗号実装ミス)を回避できる。

対象OSはmacOSのみ(v1.0.0スコープ)。Linux対応は将来のv1.1.0で別途検討する。

## Goals / Non-Goals

**Goals:**
- Tauri(Rust)プロジェクトの雛形を用意する。
- `bw serve` 子プロセスを起動・監視・終了時killするライフサイクル管理を実装する。
- `bw serve` のHTTP APIに対する薄いRustクライアントラッパー(status/unlock/lock/list items/get item/get totp)を実装する。
- vaultのロック状態をRust側のアプリ状態として保持し、後続changeのUI層が参照できるようにする。

**Non-Goals:**
- メニューバーアイコン・グローバルホットキー・ポップアップウィンドウの実装(`menubar-hotkey-shell` で扱う)。
- 検索UI・コピー/ブラウザで開くアクション・自動ロックのUXロジック(`quickaccess-search-ui` / `credential-actions-autolock` で扱う)。
- 配布パッケージング・コード署名・既存TUIの削除(`gui-distribution-and-tui-removal` で扱う)。
- Linux対応。

## Decisions

### 1. Tauri(Rust + WebView)を採用する

macOS専用であれば Swift + AppKit も選択肢だったが、v1.1.0でのLinux対応を早期に見据えているため、UI層の書き直しを避けられるTauriを採用する。tray-icon・グローバルホットキーの両方についてmacOS/Linux/Windowsをカバーするcrateがエコシステムに存在する。本changeの時点ではUIを持たないため、この決定の影響は「Rustでバックエンドを書く」という点のみに留まる。

代替案として検討したが採用しなかったもの:
- Swift + AppKit: macOS向けには最もネイティブだが、Linux移植が事実上の別実装になる。
- Electron: クロスプラットフォームだがバイナリサイズが過大(常駐メニューバーツールとして不釣り合い)。

### 2. `bw serve` はポートを動的に選択し、`localhost` にのみバインドする

固定ポートは他プロセスとの衝突や、複数インスタンス起動時の問題を招く。OSにポート0を渡して空きポートを割り当てさせ(`TcpListener::bind("127.0.0.1:0")` で一時的に確保してから解放し、そのポート番号を `bw serve --port <port>` に渡す方式、もしくは同等の空きポート検出)、アプリ内部の状態にのみ保持する。`--hostname` は明示的に `localhost` を指定し、`--hostname all` は使わない(外部ネットワークに一切晒さない)。

### 3. 子プロセス管理は Rust 標準の `std::process::Command`(非同期実行が必要なら `tokio::process::Command`)で行う

Tauriには `tauri-plugin-shell` によるシェル実行機能もあるが、任意コマンド実行の権限をフロントエンド(WebView)側に開放する用途のプラグインであり、本ユースケース(Rustコア内部で `bw serve` を1プロセスだけ管理する)には過剰。標準ライブラリの `Command` で十分であり、権限面でも余計な attack surface を増やさない。

### 4. HTTPクライアントは `reqwest`(非同期)を使う

`bw serve` のレスポンスはJSONであり、Rustエコシステムで標準的な `reqwest` + `serde_json` を採用する。各エンドポイントに対する薄いラッパー関数(`status()`, `unlock(password)`, `lock()`, `search_items(query)`, `get_item(id)`, `get_totp(id)`)をモジュール化し、後続changeのUI層(Tauri commands)から呼び出せるようにする。

### 5. ロック状態はRustのアプリ状態(`tauri::State` 相当)として保持する

`bw serve` の `/status` をポーリングするのではなく、本サービスが `/unlock` `/lock` を呼んだ結果を自身の内部状態としてキャッシュし、UI層には状態変更イベントとして通知できる形にしておく(後続changeでのメニューバーアイコン表示・自動ロックタイマーの実装土台)。

## Risks / Trade-offs

- [`bw serve` はlocalhostに認証トークンなしで立つため、アンロック中の窓の間は同一マシン上の他ローカルプロセスからも `curl` 等でvault全体を読める] → 現行のkeychainトークン方式(OSのACLで保護)より信頼境界が緩くなることを認識した上で受容する。緩和策(アイドルタイマーによる自動ロックで窓を短く保つ)は `credential-actions-autolock` で実装する。動的ポート割り当てはあくまで衝突回避目的であり、セキュリティ境界としては機能しない点に注意。
- [`bw serve` 子プロセスが予期せず終了した場合、UI層からの操作が全て失敗する] → 本changeでは子プロセスの終了を検知しアプリ状態に反映するところまでを実装し、再起動ポリシー(自動再起動 or ユーザー通知)は実装時に決定する。
- [`bw` CLIが未インストール、または `serve` を持たない古いバージョンの場合] → 既存 `lib/preflight.sh` の考え方(必要ツールの事前チェックとインストール案内)を踏襲し、本サービス起動前に `bw --version` を確認するチェックを設ける。
- [Tauri/Rustツールチェーンの追加は既存のbash/bats開発フローと並存することになり、開発環境のセットアップコストが増える] → README・CONTRIBUTING相当のドキュメント更新は `gui-distribution-and-tui-removal` でまとめて行う。

## Migration Plan

- 新規コード追加のみ(`app/` ディレクトリ以下)。既存の `bin/bw-quickaccess` / `lib/*.sh` / 既存テストへの変更はない。
- UIが存在しないため、エンドユーザーへの挙動変化は発生しない。
- ロールバックは該当コミットの revert で対応可能(データマイグレーションなし)。

## Open Questions

- `bw serve` 子プロセスの再起動ポリシー(自動リトライ回数・バックオフ)は実装時に決定する。
- Tauri / Rustのバージョン固定方針(MSRV)は実装時のCargo.toml整備時に決定する。

## 1. CSP設定の実装

- [x] 1.1 `app/src-tauri/tauri.conf.json` の `app.security.csp` を `null` から object形式のポリシーに変更する。design.mdの「採用するディレクティブと許可元」表のとおり、`default-src: ["'self'"]`, `script-src: ["'self'"]`, `style-src: ["'self'"]`, `img-src: ["'self'", "https://icons.bitwarden.net"]`, `connect-src: ["ipc:", "http://ipc.localhost"]` を設定する

## 2. ビルド確認

- [x] 2.1 `cargo build`(`app/src-tauri` ディレクトリ)でtauri.conf.jsonの変更がパースエラーなくビルドできることを確認する
- [x] 2.2 `cargo clippy --all-targets -- -D warnings` を実行し、警告が出ないことを確認する

## 3. 実機での動作確認

- [x] 3.1 開発ビルドを起動し、ブラウザDevToolsのコンソールにCSP違反(`Refused to ...`)が出ていないか確認しながら以下を一通り操作する:
  アンロック → 検索(結果表示) → ログインアイテムのアイコン表示確認 → ユーザー名/パスワード/TOTPコピー → ブラウザで開く → ロック → ユーザー確認済み、問題なし
- [x] 3.2 アイコンが表示されるログインアイテム(`icon_domain` あり)と、表示されないアイテム(プレースホルダー表示)の両方で異常がないことを確認する → ユーザー確認済み、問題なし
- [x] 3.3 3.1・3.2で問題(機能停止・CSP違反ログ)が見つかった場合は該当ディレクティブを見直し、1.1に戻って修正する → 問題なし、対応不要

## 4. ドキュメント確認

- [x] 4.1 README.mdにセキュリティ・CSPに関する既存の記述がないか確認し、あれば今回の変更を反映する(なければ対応不要) → 該当記述なし、対応不要
- [x] 4.2 `openspec/specs/` 配下に本changeで追加した `webview-csp` 以外に影響する既存specがないか再確認する(design.mdのNon-Goalsどおり、なければ対応不要) → 該当specなし、対応不要

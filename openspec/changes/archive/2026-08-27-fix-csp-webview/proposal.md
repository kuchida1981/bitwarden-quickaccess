## Why

`app/src-tauri/tauri.conf.json` の `app.security.csp` が `null` になっており、アプリのWebViewでContent-Security-Policyが完全に無効化されている。パスワードマネージャーのUIとして、インラインスクリプト実行や想定外の外部リソース読み込みに対する制限層が存在しないのは望ましくない(issue #81)。加えて、Bitwardenアイコン取得(`https://icons.bitwarden.net`)による外部通信が既に実装されており、CSP不在のままではこの通信も無制限に許可されてしまっている。

## What Changes

- `tauri.conf.json` の `app.security.csp` に、必要最小限の許可のみを与える厳格なCSPをobject形式で設定する。
  - `default-src 'self'`
  - `script-src 'self'`(Tauriが自身の初期化スクリプト用にnonce/hashを自動注入するため`unsafe-inline`は不要)
  - `style-src 'self'`(インラインstyle属性が存在しないため`unsafe-inline`は不要)
  - `img-src 'self' https://icons.bitwarden.net`(アイコン取得用)
  - `connect-src ipc: http://ipc.localhost`(Tauri v2のIPC通信 `invoke()`/`listen()` に必須)
- CSP導入後、実機(macOSアプリ)でアンロック→検索→アイコン表示→フィールドコピー→ブラウザで開く→ロックの一連の操作が壊れないことを確認する。

## Capabilities

### New Capabilities
- `webview-csp`: WebViewに適用するContent-Security-Policyの内容と、各ディレクティブを許可した理由(IPC通信・アイコン取得)を定義する。

### Modified Capabilities
(なし。既存のspecs配下に本変更の対象となる振る舞い仕様はない)

## Impact

- 影響ファイル: `app/src-tauri/tauri.conf.json`
- 影響範囲: WebView全体のリソース読み込みポリシー。フロントエンド(`app/dist/*.js`, `*.css`, `index.html`)のコード自体は変更しないが、CSP違反時は該当リソースの読み込み・実行がブロックされる。
- 依存: Tauri v2のCSP自動nonce/hash注入機構(`dangerousDisableAssetCspModification`はfalseのまま=デフォルト)。
- テスト範囲: CSPの実効性はWebView上のブラウザ動作でしか検証できないため、`cargo test`ではカバーできず実機での目視確認が必要。

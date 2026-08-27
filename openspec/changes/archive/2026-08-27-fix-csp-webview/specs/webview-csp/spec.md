## ADDED Requirements

### Requirement: WebViewはContent-Security-Policyで保護される
アプリケーションのWebViewは、`tauri.conf.json` の `app.security.csp` に設定された明示的なContent-Security-Policyの下で動作しなければならない(SHALL)。ポリシーは`null`(無効化)であってはならない。

#### Scenario: CSPが設定されている
- **WHEN** アプリが起動しWebViewが初期化される
- **THEN** WebViewには `default-src 'self'` を含むCSPが適用されており、ポリシー未指定のリソース種別は自己オリジン以外からロードされない

### Requirement: スクリプト・スタイルは自己ホストのリソースのみ許可される
CSPの `script-src` と `style-src` は `'self'` のみを許可し、`'unsafe-inline'` や `'unsafe-eval'` などインラインコード実行を広く許可するキーワードを含んではならない(SHALL NOT)。

#### Scenario: 自己ホストのスクリプト・スタイルは正常に動作する
- **WHEN** `index.html` が `i18n.js` / `app.js` / `style.css` を読み込む
- **THEN** これらは同一オリジンの自己ホストリソースとしてCSP違反なく読み込まれ、実行される

### Requirement: アイコン画像はBitwardenアイコンサービスからのみ許可される
CSPの `img-src` は `'self'` に加えて、ログインアイテムのファビコンを取得するために `https://icons.bitwarden.net` を許可しなければならない(SHALL)。それ以外の任意ドメインからの画像読み込みは許可されない。

#### Scenario: ログインアイテムのアイコンが表示される
- **WHEN** 検索結果にログインアイテムが表示され、`icon_domain` が設定されている
- **THEN** `https://icons.bitwarden.net/<icon_domain>/icon.png` への画像リクエストがCSP違反なく実行され、アイコンが表示される

#### Scenario: アイコン取得に失敗してもプレースホルダーで代替される
- **WHEN** アイコン画像の読み込みに失敗する(ネットワークエラー、または`icon_domain`が未設定)
- **THEN** アプリはプレースホルダー表示にフォールバックし、機能全体は継続動作する

### Requirement: TauriのIPC通信はCSPの下で許可される
CSPの `connect-src` は、Tauri v2のIPC機構(`invoke()` / `listen()`)が使用する `ipc:` スキームおよび `http://ipc.localhost` を許可しなければならない(SHALL)。

#### Scenario: バックエンドコマンド呼び出しが機能する
- **WHEN** フロントエンドが `invoke("unlock", ...)` や `invoke("search_items", ...)` などのTauriコマンドを呼び出す
- **THEN** CSPによってIPC通信がブロックされることなく、コマンドが実行され結果が返る

#### Scenario: バックエンドイベント購読が機能する
- **WHEN** フロントエンドが `listen()` でバックエンドからのイベントを購読する
- **THEN** CSPによってイベント通知がブロックされることなく、フロントエンドに届く

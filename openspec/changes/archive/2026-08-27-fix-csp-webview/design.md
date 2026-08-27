## Context

`app/src-tauri/tauri.conf.json` の `app.security.csp` は現在 `null` で、CSPが無効化されている(issue #81)。フロントエンドは `app/dist/` 配下の静的ファイル(`index.html`, `app.js`, `i18n.js`, `style.css`)で、ビルドツールを経由しない。

調査の結果、以下が判明している:
- インラインの `<script>` タグやインライン `style=""` 属性は存在しない。すべて `<script src>` 経由で外部ファイルを読み込んでいる。
- JavaScriptから `element.style.property = value` の形でスタイルを直接変更している箇所があるが(例: `app.js` の `emptyMessage.style.display`)、これはCSSOM経由の操作でありCSPの `style-src` の対象外(ブラウザはこれを「インラインstyle」として扱わない)。
- 唯一の外部ネットワークアクセスは `app.js` の `iconImg.src = \`https://icons.bitwarden.net/${domain}/icon.png\`` で、ログインアイテムのアイコンを取得している。`icon_domain` はログインアイテムのURIから動的生成されるが(`http_client.rs`)、リクエスト先ホスト自体は常に固定の `icons.bitwarden.net`。
- アプリの全バックエンド通信は Tauri v2 の `invoke()` / `listen()` によるIPCで行われる。Tauri公式ドキュメントによれば、このIPCはCSPの `connect-src` の管理下にあり、`ipc:` と `http://ipc.localhost` の許可が必要(プラットフォームによってどちらの経路を使うか異なるため両方指定するのが公式の推奨)。
- Tauriはビルド時に自身が注入する初期化スクリプト用のnonce/hashを自動的にCSPへ追加するため(`dangerousDisableAssetCspModification: false` がデフォルト)、`script-src 'self'` のみで追加のnonce設定は不要。

## Goals / Non-Goals

**Goals:**
- WebViewに、実際に使用しているリソース読み込みパターンだけを許可する最小権限のCSPを設定する。
- CSP導入によってアプリの既存機能(アンロック、検索、フィールドコピー、アイコン表示、ブラウザで開く、ロック)が壊れないことを保証する。

**Non-Goals:**
- フロントエンドのコード自体(`app.js` 等)の変更は行わない。CSPポリシー設定のみが対象。
- 将来的な外部リソース追加(例: 自己ホストVaultwardenサーバーごとに異なるアイコンドメインなど)への対応は本changeのスコープ外。必要になった時点で別途対応する。

## Decisions

### CSPの記述形式: object形式を採用
`tauri.conf.json` の `csp` はstring形式(`"default-src 'self'; script-src 'self'; ..."`)でもobject形式(`{ "default-src": ["'self'"], ... }`)でも設定できる。本changeではobject形式を採用する。

理由: ディレクティブごとに許可元が配列で分離されるため、なぜそのソースを許可しているか(IPC用、アイコン取得用など)が構造的に明確になり、将来ディレクティブを追加・変更する際の差分が読みやすい。string形式は1行にまとまる分、意図をコメントで補足できず可読性が落ちる。

### 採用するディレクティブと許可元

| ディレクティブ | 許可元 | 理由 |
|---|---|---|
| `default-src` | `'self'` | それ以外の未指定ディレクティブに対するフォールバック制限 |
| `script-src` | `'self'` | `index.html` からのスクリプト読み込みは `i18n.js`/`app.js` の自己ホストファイルのみ。Tauriが自身の初期化スクリプト分のnonce/hashを自動注入するため追加設定不要 |
| `style-src` | `'self'` | `style.css` の自己ホストのみ。インラインstyle属性なし、CSSOM経由の `.style.xxx` はCSP対象外のため `unsafe-inline` 不要 |
| `img-src` | `'self' https://icons.bitwarden.net` | プレースホルダーアイコン等は`'self'`、ログインアイテムのファビコン取得は固定ドメイン `icons.bitwarden.net` |
| `connect-src` | `ipc: http://ipc.localhost` | Tauri v2のIPC(`invoke`/`listen`)経路。公式推奨どおり両プロトコルを許可 |

`script-src`/`style-src`/`img-src`/`connect-src` を明示するため、`default-src 'self'` は主に `frame-src`, `font-src`, `media-src` など未使用機能への保険として機能する。

### 代替案として検討したが採用しなかったもの
- **string形式でissueのサンプルをそのまま採用**(`style-src 'self' 'unsafe-inline'`): 実際にはインラインstyleが存在せず、`unsafe-inline` は不要な権限拡大になるため不採用。
- **`connect-src` を省略**: IPCが `connect-src` の対象であることを見落とすと、アンロックや検索などIPC呼び出しを伴う機能がすべて動作しなくなる。実機確認で必ず検出できるが、事前調査で分かっている問題を後回しにする理由がないため明示的に含める。

## Risks / Trade-offs

- [Risk] `connect-src` にIPC許可を入れ忘れる、または誤ったスキームを指定すると、CSPエラーはブラウザのDevToolsコンソールにしか出ず、アプリはビルド・起動できてしまうため気づきにくい(アンロックボタンを押しても無反応になる等の形で顕在化する) → Mitigation: 実機での動作確認タスク(アンロック→検索→アイコン表示→コピー→ブラウザで開く→ロック)を tasks.md に必須項目として含める。
- [Risk] `cargo test` はCSPの実効性(WebView上での実際のブロック挙動)を検証できない → Mitigation: 上記と同じく実機確認で代替する。自動テストでの担保は本changeのスコープでは行わない。
- [Trade-off] `icons.bitwarden.net` をハードコードで許可することで、将来的にアイコン取得先が変わった場合(例: 自己ホストサーバーの独自アイコンエンドポイント)は追加のCSP変更が必要になる。現状の実装が固定ドメインしか使っていないため、現時点では過剰な一般化を避けこの制約を受け入れる。

## Migration Plan

`tauri.conf.json` の設定変更のみで、データ移行やロールバック手順は不要。CSP導入によって既存機能に問題が出た場合は、該当ディレクティブに許可元を追加するか、問題が大きければ `csp: null` に戻す(ロールバック)ことで即座に復旧できる。

## Open Questions

なし。

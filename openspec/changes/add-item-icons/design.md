## Context

現在 `bw serve` から取得するアイテム情報は `VaultItemSummary`(`app/src-tauri/src/backend/http_client.rs`)としてパースされており、`id` / `name` / `login`(`username` / `password` / `totp` / `uris`)のみを保持する。`serde` はデフォルトで未知のJSONフィールドを無視するため、実際のレスポンスにアイコン関連フィールドが含まれていたとしても、現状のパース処理ではそれを検知できず、単に破棄されている。

**【調査結果・確定】** ユーザーが実機の `bw serve` (`/list/object/items`) を叩いて確認した結果、アイテムのレスポンスには `type` / `name` / `favorite` / `reprompt` / `id` / `collectionIds` / `folderId` / `fields` / `login`(`uris`(各要素は `uri` キーのみ)/ `fido2Credentials` / `username` / `password` / `totp` / `passwordRevisionDate`)/ `passwordHistory` / `creationDate` / `revisionDate` / `attachments` のみが含まれ、アイコン画像・アイコンURLに相当するフィールドは存在しないことが確認された。したがって本changeは **パスB(ドメイン抽出+外部アイコン取得)** で実装する。

## Goals / Non-Goals

**Goals:**
- ドメイン抽出+外部アイコン取得により、検索結果一覧の各行にアイコン(またはプレースホルダー)を表示する。
- アイコン取得の遅延・失敗がクイックアクセスの即応性(検索・行フォーカス移動)を損なわないようにする。

**Non-Goals:**
- アイコンの手動アップロード・カスタムアイコン設定などBitwarden本体側の機能拡張は行わない。
- フォルダ・組織単位のアイコン等、アイテム個別以外のアイコン表示は扱わない。
- アイコン取得先サービスの選定を独自に評価・比較検討する詳細な調査は本designのスコープ外とし、以下で候補とする1案を採用する。

## Decisions

### アイコン取得先サービス
Bitwardenの公式クライアント(Web Vault等)自体が、アイテムのドメインを基に自社のアイコン取得サービス(`https://icons.bitwarden.net/{domain}/icon.png` 形式)へ問い合わせてファビコンを表示する方式を採っている(公開情報に基づく想定)。本changeでもこれと同じエンドポイントを第一候補とする。理由:
- 既に公式クライアントが常用しているサービスであり、新規の第三者サービスへの信頼を追加する必要がない。
- 実装がシンプル(ドメイン抽出のみで済み、APIキー等の追加設定が不要)。

ただし実在するエンドポイント形式・レスポンス仕様は実装時タスク(3.1)で実機確認し、想定と異なる場合はここを更新する。

### ドメイン抽出
- `login.uris[0].uri` からホスト名を抽出する。`uris` が空、または `login` 自体が無いアイテムはアイコン関連情報なし(プレースホルダー表示)として扱う(`vault-backend-service` の要件通り)。
- 抽出処理はRust側(`http_client.rs` または `commands.rs`)に実装し、`SearchResultItem` に `icon_domain: Option<String>` として持たせる。

### アイコン取得元(フロントエンド/バックエンド)
アイコン画像の実際の取得(HTTP GET)はフロントエンド(`app/dist/app.js`)から `<img src="https://icons.bitwarden.net/{domain}/icon.png">` の形で直接行う。理由:
- `tauri.conf.json` の `security.csp` は現状 `null`(無制限)であり、追加のCSP許可設定なしに即座に動作する。
- Rust側でプロキシする場合と比べ、画像データをTauriのIPC経由でやり取りする必要がなく、実装・パフォーマンスの両面でシンプル。
- `<img>` の `onerror` でプレースホルダーへのフォールバックをブラウザ標準機構だけで完結できる。

### キャッシュ方針
アイコンの取得はブラウザの標準HTTPキャッシュ(`<img>` タグによる通常の画像リクエスト)に委ね、アプリ側で独自のメモリ内キャッシュは持たない。理由: 同一セッション内で同じドメインの `<img src>` を複数回描画してもブラウザキャッシュが効くため、独自実装を追加する必要性が薄い。永続キャッシュ(ディスク保存)は本changeのスコープ外とする。

### 表示コンポーネントの実装場所
アイコンの表示・プレースホルダーへのフォールバックは `app/dist/app.js` の `renderResults` に実装する。`<img>` の `onerror` ハンドラでプレースホルダーへの切り替えを行う。

## Risks / Trade-offs

- [外部サービス(`icons.bitwarden.net` 想定)へのドメイン情報の送信自体がプライバシー上の懸念になり得る(アクセス先ドメインがどのアイテムを見ているかの手がかりになる)] → Bitwarden公式クライアント自体が同じ方式を採用している(想定)ため新規のプライバシーリスクの追加ではないと考えられるが、`/security-review` で最終確認する。将来的に設定でアイコン表示を無効化できる余地を残す(本changeでは設定UIの追加までは行わない)。
- [アイコン読み込みの遅延によりポップアップの即応性が損なわれる] → `<img>` の非同期読み込みは一覧描画・行フォーカス移動をブロックしない(`incremental-item-search` の要件で担保)。
- [想定したアイコン取得エンドポイントが実在しない、または想定と異なる形式だった場合] → 実装タスク(3.1)で実機確認し、design.mdを更新の上で代替案を検討する。

## Open Questions

- `https://icons.bitwarden.net/{domain}/icon.png` 形式のエンドポイントが実際に想定通り機能するかは実装タスク(3.1)で確認する。

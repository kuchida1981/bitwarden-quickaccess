## Why

検索結果の一覧がテキストのみで構成されており、似た名前のアイテムやアカウントが並ぶ際に視認性が低い。1Passwordクイックアクセスのようにファビコン風アイコンを各行に表示することで、目的のアイテムを素早く見分けられるようにしたい(Issue #69)。実機で `bw serve` のAPIレスポンスを確認した結果、アイコン関連情報は含まれていないことが確認できた(`design.md` 参照)ため、ドメインベースの外部アイコン取得方式で実装する。

## What Changes

- アイテムの `login.uris` からドメインを抽出し、`SearchResultItem` に含めるようにする。
- ドメインを用いて外部アイコン取得(Bitwarden公式クライアントと同じアイコン取得サービスを想定)を行う。フロントエンド(`app/dist/app.js`)から `<img>` タグで直接取得する。
- 検索結果一覧の各行(`app/dist/app.js` の `renderResults`)にアイコン(またはプレースホルダー)を表示する。
- アイコン取得に失敗した場合・ドメイン情報が無い場合のフォールバック表示(プレースホルダーアイコン)を用意する。

## Capabilities

### New Capabilities
(なし)

### Modified Capabilities
- `vault-backend-service`: アイテム検索・取得インターフェースが返すデータに、アイコン取得に必要なドメイン情報を含めるようになる。
- `incremental-item-search`: 検索結果一覧の行要約表示にアイコン(またはプレースホルダー)が加わる。

## Impact

- `app/src-tauri/src/backend/http_client.rs` / `app/src-tauri/src/commands.rs`: `SearchResultItem` へのドメイン抽出・伝播処理。
- `app/dist/app.js` (`renderResults`): 行描画へのアイコン追加(外部アイコンサービスへの `<img>` リクエスト、`onerror`によるプレースホルダーへのフォールバック)。
- `app/dist/style.css`: アイコン用のレイアウト・プレースホルダースタイル。
- 外部アイコンサービスへの通信が発生するため、`/security-review` での確認対象となる。

## Why

現在ショートカットヒントは選択中の行の中に埋め込まれており、折り返して一覧の見た目を圧迫している(Issue #64)。また、ロック状態でクイックアクセスを表示しても、どのBitwardenアカウントでログイン中かを確認する手段がない(Issue #70)。1Passwordクイックアクセスに倣い、この2つをポップアップ下部の共通のグローバル領域(フッタ)にまとめて表示する。

## What Changes

- 検索画面の各行からショートカットヒントの埋め込み表示を削除する。
- 検索画面下部に常時表示のフッタ領域を新設し、左側にログイン中アカウントの頭文字アバター(hoverで完全なメールアドレスを表示)、右側にショートカットヒント(固定文言)を表示する。
- `bw serve` の `/status` レスポンスに含まれる `userEmail` を(現状は破棄されている)パースし、フロントエンドに渡せるようにする。
- アクションメニュー展開中の各アクション行のヒント(`action-hint`)表示は現状維持する(フッタのヒントと役割が重複しないよう、フッタは行操作全般の固定ヒント、アクションメニューは選択中アクションの実行キーという整理のまま変更しない)。
- フッタ(アカウント表示+ヒント)を横1行で収めるため、ポップアップウィンドウの幅を現状の420pxから広げる(1Passwordクイックアクセスの横幅を参考にする、Issue #64のユーザーコメント)。

## Capabilities

### New Capabilities
- `quickaccess-status-footer`: 検索画面下部の常時表示フッタ(ログイン中アカウント表示・グローバルショートカットヒント表示)を提供する。

### Modified Capabilities
- `vault-backend-service`: 状態取得インターフェースの戻り値に、ログイン中アカウントのメールアドレスを含めるようになる。
- `incremental-item-search`: 行フォーカス時に行内へショートカットヒントを表示する要件を廃止する(`quickaccess-status-footer` に統合されるため)。

## Impact

- `app/src-tauri/src/backend/http_client.rs`: `StatusTemplate` へのフィールド追加、`status()` の戻り値変更。
- `app/src-tauri/src/backend/state.rs`: `AppState` にログイン中アカウントのメールアドレスを保持するフィールド・アクセサを追加。
- `app/src-tauri/src/main.rs`: `sync_initial_status` での反映。
- `app/src-tauri/src/commands.rs`: アカウント取得用の新規コマンド追加。
- `app/dist/index.html` / `app/dist/style.css`: フッタ領域のマークアップ・スタイル追加、行内ヒント表示の削除。
- `app/dist/app.js`: `buildTrailingBlock` から行内ヒント生成を削除、フッタへのアカウント表示・ヒント表示の反映処理を追加。
- `app/src-tauri/src/popup.rs`: ポップアップウィンドウの `WIDTH` 定数を拡大する。

## 1. 実現可否調査(完了)

- [x] 1.1 (ユーザー作業)実機の `bw serve` (`/list/object/items`) レスポンスを確認。アイコン関連フィールドは存在せず、ドメイン抽出+外部アイコン取得(パスB)が必要と判明。
- [x] 1.2 `design.md` にパスB確定の決定事項を反映。

## 2. バックエンド実装: ドメイン抽出

- [x] 2.1 `app/src-tauri/src/backend/http_client.rs` に `LoginDetail::icon_domain()` を実装(agy, commit 8234aa1)。
- [x] 2.2 `app/src-tauri/src/commands.rs` の `SearchResultItem` に `icon_domain: Option<String>` を追加し、`From<VaultItemSummary>` に反映(agy, commit 8234aa1)。
- [x] 2.3 `icon_domain` のテストを追加(agy, commit 8234aa1)。

## 3. フロントエンド実装: アイコン取得・表示

- [x] 3.1 実装前に、想定エンドポイント(`https://icons.bitwarden.net/{domain}/icon.png` 形式)が実際に画像を返すか確認する。(2026-08-27 `curl` で `https://icons.bitwarden.net/amazon.co.jp/icon.png` を確認し、`200 image/png` を確認済み)
- [x] 3.2 `app/dist/app.js` の `renderResults` に `icon_domain` を持つアイテムのアイコン表示を追加(agy, commit be7e24f)。
- [x] 3.3 `<img>` の `onerror` によるプレースホルダーへのフォールバックを実装(agy, commit be7e24f)。
- [x] 3.4 アイコン読み込みは非同期(`<img>` 標準の読み込みのみ、同期待ちなし)であることを確認(agy, commit be7e24f)。
- [x] 3.5 `app/dist/style.css` にアイコン・プレースホルダーのスタイルを追加(agy, commit be7e24f)。

## 4. セキュリティレビュー

- [ ] 4.1 `/security-review` を実行し、外部アイコンサービスへの通信追加によるセキュリティ・プライバシー上の懸念がないか確認する。指摘があれば対応する。

## 5. 動作確認

- [x] 5.1 `cargo test` を実行し、既存テストおよび追加したテストが通ることを確認する。(2026-08-27 全12テスト成功)
- [x] 5.2 `cargo clippy --all-targets -- -D warnings` を実行し、警告が無いことを確認する。(2026-08-27 警告なし)
- [ ] 5.3 実機で、URIを持つアイテム・持たないアイテムそれぞれについて、一覧行にアイコン(またはプレースホルダー)が表示されることを確認する。
- [ ] 5.4 実機で、オフライン状態(またはアイコン取得先に到達できない状態)でもクイックアクセスの検索・行フォーカス移動が問題なく行え、プレースホルダーが表示されることを確認する。

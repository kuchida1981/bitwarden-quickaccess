## Why

GUI版(`app/`)のUI文言は、メニューバーのトレイメニュー(`app/src-tauri/src/tray.rs`)・検索ポップアップ(`app/dist/app.js` / `index.html`)ともに日本語ハードコードで、README.md の `## Out of scope` にも「In-app localization (the UI text is currently Japanese-only; no language switching)」と明記されている。英語話者のユーザーが利用しづらく、v1.1.0 に向けて予定している他の1Password挙動追従の機能追加(rightキーのアクションメニュー、ヘルプ表示、バージョン情報UI等)が新たに文言を追加する前に、辞書ベースの仕組みへ移行しておきたい。基盤を先に用意することで、後続機能は最初から辞書経由で文言を書け、あとから翻訳を洗い出す手戻りを避けられる。

なお、旧ターミナル版(`bin/bw-quickaccess` / `lib/*.sh`)向けに同種のi18n対応が過去に一度実装されていた(`openspec/changes/archive/2026-08-07-add-i18n-support/`)が、GUIへの書き換え(`2026-08-26-gui-distribution-and-tui-removal`)でその実装ごと削除済みであり、現行コードには存在しない。判定順序の考え方(明示指定 → OSロケール → フォールバック)は踏襲するが、実装機構はTauri/Rust + vanilla JSのGUI向けに新規に作る。

## What Changes

- Rustネイティブのトレイメニュー文言(ステータス表示・ホットキー警告・自動起動トグル・バージョン表示・終了 等、`app/src-tauri/src/tray.rs`)を、言語別の文言テーブル参照に置き換える
- Webビュー側のポップアップUI文言(検索プレースホルダ、アンロック画面の案内・エラー、フィードバックメッセージ「ユーザー名をコピーしました」等、`app/dist/app.js` / `app/dist/index.html`)を、言語別の辞書(JSONまたはJSオブジェクト)参照に置き換える
- 言語判定ロジックを実装する: 明示指定(環境変数等) → OSのロケール設定 → 対応言語が無ければ英語(`en`)にフォールバック。Rust側・JS側それぞれで判定結果を参照できるようにする(起動時にRust側で判定し、フロントエンドには判定結果を渡す想定)
- 対応言語は日本語(`ja`)・英語(`en`)の2言語とする
- README.md / README.ja.md の `## Out of scope` から「In-app localization」の記述を削除し、実態に合わせて更新する
- 後続で追加予定のUI(rightキーのアクションメニュー、ヘルプ表示、バージョン情報UI等)がこの辞書機構にそのまま乗せられるよう、文言キーの追加が容易な構造にする(このchange自体はそれらの新規UIを実装しない)

## Capabilities

### New Capabilities
- `ui-localization`: アプリのUI文言(トレイメニュー・検索ポップアップ・エラーメッセージ等)を日本語・英語で切り替え可能にし、明示指定またはOSロケールに応じて実行時に言語を選択する

### Modified Capabilities
(なし。既存capability(`menubar-presence`, `global-hotkey-popup` 等)の振る舞い自体は変更せず、表示文言の言語のみが変わる)

## Impact

- 影響コード: `app/src-tauri/src/tray.rs`(メニュー文言)、`app/dist/app.js` / `app/dist/index.html`(ポップアップUI文言)、`app/src-tauri/src/main.rs` または `lib.rs`(起動時の言語判定・フロントエンドへの受け渡し)
- 新規ファイル: 言語別の文言テーブル(Rust側・JS側それぞれに新設。具体的な配置は design.md で決定)
- ドキュメント: README.md / README.ja.md(Out of scope 記述の更新)
- 新規外部依存: 未定(design.mdで既存クレート/軽量自前実装のいずれにするか検討する)
- 後方互換性: 環境変数等の明示指定が無い場合はOSロケールで判定するため、既存ユーザー(日本語ロケール環境)の見た目は変わらない
- 本changeの対象外: rightキーのアクションメニュー・ヘルプ表示・バージョン情報UIなど、まだ実装されていないUIの文言追加(それぞれ別issue/changeで対応し、実装時にこの辞書機構へ文言を追加する)

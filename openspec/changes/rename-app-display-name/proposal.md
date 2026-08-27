## Why

ロック状態でクイックアクセスを表示すると "bw-quickaccess" という内部的なプロダクト名がそのまま表示される(ウィンドウタイトル、アンロック画面の見出し)。実際のアプリ名として無機質で親しみが薄い(Issue #68)。ユーザーとの相談の結果、`.app` バンドル名・Homebrew配布に関わる識別子(`productName`/バンドルID)は変更せず、UI上の表示文言のみを親しみやすい名称に変更し、既存の日英ローカライズの枠組みに乗せる。

## What Changes

- ユーザーに見せる表示名を「Bitwarden クイックアクセス」(日本語)/「Bitwarden Quick Access」(英語)に変更する。
- `app/dist/index.html` の `<title>` とアンロック画面の `<h1>` を、既存の `data-i18n` 属性の仕組みに乗せてローカライズする(`app/dist/i18n.js` に新規キーを追加)。
- `app/src-tauri/src/popup.rs` のネイティブウィンドウタイトル(`.title("bw-quickaccess")`)を、`i18n::Messages` に追加する新規フィールド経由でローカライズされた表示名に変更する。
- `tauri.conf.json` の `productName` / バンドルID(`com.kuchida1981.bw-quickaccess`)、トレイメニューの About 項目(`app.package_info().name` に依存)は変更しない(Homebrew配布・自動更新への影響を避けるため)。

## Capabilities

### New Capabilities
(なし)

### Modified Capabilities
- `ui-localization`: アプリの表示名(ウィンドウタイトル・アンロック画面の見出し)が、表示言語に応じてローカライズされるようになる。

## Impact

- `app/dist/index.html`: `<title>` / `<h1>` への `data-i18n` 属性追加。
- `app/dist/i18n.js`: 新規キー(表示名の日英文言)追加。
- `app/src-tauri/src/i18n.rs`: `Messages` への新規フィールド追加。
- `app/src-tauri/src/popup.rs`: ウィンドウタイトルのローカライズ対応。

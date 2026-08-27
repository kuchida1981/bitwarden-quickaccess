## Context

`bw-quickaccess` という内部プロダクト名が、以下3箇所にハードコードされてユーザーに見えている:
- `app/dist/index.html:5` の `<title>bw-quickaccess</title>`
- `app/dist/index.html:10` のアンロック画面 `<h1>bw-quickaccess</h1>`
- `app/src-tauri/src/popup.rs:28` の `WebviewWindowBuilder` に対する `.title("bw-quickaccess")`(なお当該ウィンドウは `.decorations(false)` でOS標準のタイトルバー自体を表示しないため、この値がユーザーに直接見える場面は限定的だが、Mission Control等での識別に使われうるため統一する)

ユーザーとの相談の結果、`tauri.conf.json` の `productName`(Finder上の `.app` 表示名の元)やバンドルID、トレイメニューAbout項目(`app.package_info().name` 依存、`about-and-branding` capability管轄)は変更しない。UI表示テキストのみを、既存のローカライズ機構(`app/dist/i18n.js` の `data-i18n` 属性、`app/src-tauri/src/i18n.rs` の `Messages`)に乗せて日英対応させる。

## Goals / Non-Goals

**Goals:**
- ウィンドウタイトルとアンロック画面の見出しを、日英ローカライズされた親しみやすい表示名(「Bitwarden クイックアクセス」/「Bitwarden Quick Access」)に変更する。

**Non-Goals:**
- `tauri.conf.json` の `productName` / バンドルID / リポジトリ名 / バイナリ名の変更(Homebrew配布・自動更新への影響を避けるため)。
- トレイメニューのAbout項目(`app.package_info().name` を使用、`about-and-branding` capability管轄)の表示変更。

## Decisions

### 表示名の文言
「Bitwarden クイックアクセス」(日本語)/「Bitwarden Quick Access」(英語)とする。Bitwardenのツールであることが一目でわかり、既存コード内のコメントや設計文書で頻出する「クイックアクセス」という呼称とも一貫する。

### HTML側(`index.html` / `i18n.js`)
既存の `data-i18n` 属性の仕組み(`app/dist/i18n.js` の `applyStaticI18n()`)をそのまま使う。`<title>` 要素は `document.querySelectorAll("[data-i18n]")` の対象になる(`<title>` に `data-i18n` 属性を付けても問題なく動作する)。新規キー `appDisplayName` を `MESSAGES.ja` / `MESSAGES.en` に追加し、`<title data-i18n="appDisplayName">...</title>` および `<h1 data-i18n="appDisplayName">...</h1>` とする。

### Rust側(`popup.rs` / `i18n.rs`)
`i18n::Messages` に `pub app_display_name: &'static str` を追加する。`create_popup_window(app: &AppHandle)` 内で `app.state::<crate::i18n::Lang>()` を参照し(`main.rs` で `.manage(lang)` が `create_popup_window` 呼び出し(`.setup()` 内)より前に行われているため取得可能)、`crate::i18n::messages(lang).app_display_name` を `.title(...)` に渡す。

## Risks / Trade-offs

- [ネイティブウィンドウタイトルは `.decorations(false)` によりOS標準のタイトルバーとしては表示されないため、この変更の体感上の効果は限定的] → それでもMission Control等での識別や将来の変更に備え、HTML側と一貫させておく価値はあると判断する。

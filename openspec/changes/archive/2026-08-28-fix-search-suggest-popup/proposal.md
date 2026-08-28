## Why

検索ボックスに文字を入力すると、macOSのインラインテキスト候補(スペルチェック/オートコレクト/オートキャピタライズ)のポップアップが表示されてしまう(#111)。このポップアップが表示されている間は↑/↓キーがポップアップ側に奪われ、検索結果一覧のキーボードによる行フォーカス移動ができなくなる。1Password Quick Access相当の体験を目指す本アプリにとって、このような不要なOSネイティブ候補表示は目的にそぐわない。

## What Changes

- `app/dist/index.html` の `#search-box` input要素に `spellcheck="false"`、`autocorrect="off"`、`autocapitalize="off"` を追加する(既存の `autocomplete="off"` はそのまま維持)
- 検索ボックス入力中・入力直後でも↑/↓キーによる行フォーカス移動がOSネイティブ候補表示に妨げられないようにする

## Capabilities

### New Capabilities

(なし)

### Modified Capabilities

- `incremental-item-search`: 検索ボックスがOSネイティブのテキスト入力支援(スペルチェック/オートコレクト/オートキャピタライズ/オートコンプリート)を提供してはならない、という要件を追加する

## Impact

- 影響ファイル: `app/dist/index.html`(input要素の属性追加のみ)
- `app/dist/app.js` 側のロジック変更は不要
- 関連issue #128(スクロール起因のフォーカス誤爆)は別changeで扱う

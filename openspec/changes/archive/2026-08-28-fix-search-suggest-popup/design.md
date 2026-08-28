## Context

`app/dist/index.html` の `#search-box` input要素は現在 `autocomplete="off"` のみを指定しており、`spellcheck` / `autocorrect` / `autocapitalize` 属性は未指定(ブラウザ・OSのデフォルト挙動に依存)。そのためmacOSのWKWebView上でインラインテキスト候補ポップアップが表示され、↑/↓キー操作が競合する(#111)。

`app/dist/` はビルドチェーンなしの素のHTML/CSS/JSであり(`app/README.md`)、この修正は当該input要素へのHTML属性追加のみで完結する。

## Goals / Non-Goals

**Goals:**
- 検索ボックスでmacOSのインラインサジェスト/スペルチェック/オートコレクト/オートキャピタライズを表示させない
- 既存の `autocomplete="off"` を維持する

**Non-Goals:**
- 検索結果一覧のフォーカス移動ロジック自体の変更(#128で別途対応)
- IME(日本語入力等)の変換候補の抑制。これはOS/IME側の変換UIであり、`spellcheck`/`autocorrect`等の対象外であるため本changeのスコープ外とする

## Decisions

- **属性追加のみで対応する**: `spellcheck="false"`、`autocorrect="off"`、`autocapitalize="off"` をinput要素に静的に追加する。JavaScript側でのフォーカスイベント制御やCSSでの視覚的な非表示化は行わない。
  - 代替案として検討したがconstellationから外した案: `input`要素の`type`を変更する、キーイベントのリスニング方法を変える等 — いずれも属性追加より複雑で、ブラウザ標準の抑制機構(HTML属性)で解決できる問題に対して過剰。

## Risks / Trade-offs

- [Risk] `autocorrect`/`autocapitalize` はWebKit系(Safari/WKWebView)固有の非標準属性であり、他エンジンでは無視される → [Mitigation] 本アプリはmacOS専用(WKWebViewベースのTauri)であるため実害なし。`spellcheck`/`autocomplete`は標準属性。

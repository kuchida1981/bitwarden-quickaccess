## Context

`app/dist/app.js` の `searchBox` の `keydown` ハンドラは、現状(`quickaccess-action-menu` 実装後)以下の優先順位になっている。

```javascript
searchBox.addEventListener("keydown", (event) => {
  if (actionMenuOpen) {
    handleActionMenuKeydown(event);
    return;
  }
  if (event.key === "ArrowDown") { ... }
  if (event.key === "ArrowUp") { ... }
  if (event.key === "ArrowRight") { ... }
  handleActionShortcut(event);
});
```

`handleActionMenuKeydown` は `ArrowLeft` / `Escape` でメニューを閉じる処理を持つ。今回追加する「ヘルプ表示中」という新しい状態は、このどちらよりも外側(優先度が高い)レイヤーとして追加する必要がある。

`hide_popup` コマンド(`app/src-tauri/src/commands.rs`)は既にポップアップの非表示と直前アプリへのフォーカス復帰(`popup::restore_previous_focus`、`quickaccess-window-focus` で実装済み)を行っており、コピー操作後のフィードバック表示後にも使われている。`Esc` での即時クローズもこれをそのまま呼べばよく、Rust側の変更は不要。

## Goals / Non-Goals

**Goals:**
- `Esc` キーでポップアップを閉じられるようにする(閉じた際の直前アプリへのフォーカス復帰は既存の `hide_popup` の挙動をそのまま使う)
- `⌘/` で、現在実装済みのショートカットの一覧をポップアップ内にオーバーレイ表示する
- ヘルプ表示中・アクションメニュー表示中・通常時、という3つの状態の間でキー入力の優先順位を矛盾なく整理する

**Non-Goals:**
- ヘルプの内容編集・キーバインドのカスタマイズ機能
- ヘルプに、本アプリが実装していない1Password本家の機能(自動入力、アカウント切り替え等)を載せること
- ヘルプオーバーレイ内でのスクロール位置記憶等の凝ったUX(単純に開閉できれば十分)

## Decisions

### 1. ヘルプ表示のトリガーキーは `⌘/`

1Password Quick Accessに合わせる。`event.code === "Slash"` かつ `event.metaKey`(`shiftKey`/`altKey` は不可)で判定する。既存の `⌘C` 系判定と同様、`event.key` ではなく `event.code`(物理キー位置)で判定し、キーボードレイアウトやOption合成文字の影響を受けないようにする。既存のショートカット(`⌘C`/`⇧⌘C`/`⌥⌘C`/`Enter`/矢印キー)と衝突しないことを確認済み。

### 2. ヘルプの表示方法: `#search-screen` 内の静的HTMLオーバーレイ

ヘルプの内容は動的に変化しない(検索結果やアイテムに依存しない固定テキスト)ため、`quickaccess-action-menu` のアクションメニューのようにJSで動的構築するのではなく、`app/dist/index.html` に静的な `<div id="help-overlay">` を追加し、`data-i18n` 属性で日英を切り替える(`ui-localization` で確立した既存パターンをそのまま使う)。表示/非表示はCSSクラスの切り替え(`visible`)で行い、`#search-screen` に対して絶対配置で重ねる。

**代替案: ヘルプ内容もJSで動的生成する** — 却下。内容が固定なので動的生成のメリットがなく、静的HTML+data-i18nの方がシンプルで他の静的テキストとの一貫性もある。

### 3. キー入力の優先順位: ヘルプ表示中 > アクションメニュー表示中 > 通常

`searchBox` の `keydown` ハンドラの先頭分岐を以下の順序にする。

```javascript
searchBox.addEventListener("keydown", (event) => {
  if (helpOpen) {
    handleHelpKeydown(event);
    return;
  }
  if (actionMenuOpen) {
    handleActionMenuKeydown(event);
    return;
  }
  if (event.key === "Escape") {
    event.preventDefault();
    invoke("hide_popup").catch(() => {});
    return;
  }
  if (event.metaKey && event.code === "Slash" && !event.shiftKey && !event.altKey) {
    event.preventDefault();
    openHelp();
    return;
  }
  // 既存の ArrowDown/ArrowUp/ArrowRight/handleActionShortcut
});
```

`handleHelpKeydown` は `Escape` または `⌘/` でヘルプを閉じ、それ以外のキーはすべて無視する(`quickaccess-action-menu` の `handleActionMenuKeydown` 末尾と同様、未処理なら `preventDefault()` して検索文字入力等を抑止する)。

この順序により:
- ヘルプ表示中の `Esc` → ヘルプを閉じる(ポップアップは閉じない)
- アクションメニュー表示中(ヘルプは非表示)の `Esc` → メニューを閉じる(既存動作、変更なし)
- どちらも表示していない状態の `Esc` → ポップアップを閉じる(新規)

### 4. ヘルプを開く際、開いているアクションメニューは閉じる

`openHelp()` の冒頭で `actionMenuOpen` が `true` であれば `closeActionMenu()` を呼んでからヘルプを開く。2つの重なるUI(アクションメニューとヘルプオーバーレイ)が同時に見える状態を作らないための単純なガード。

### 5. ヘルプ本文は、現在実装済みのショートカットのみを列挙する

| キー | 説明 |
|---|---|
| `⇧⌘Space` | クイックアクセスを表示/非表示(グローバルホットキー、参考として掲載) |
| `↑` / `↓` | アイテム間を移動 |
| `→` | アクションメニューを開く |
| `←` | アクションメニューを閉じる |
| `⌘C` | ユーザー名をコピー |
| `⇧⌘C` | パスワードをコピー |
| `⌥⌘C` | ワンタイムパスワードをコピー |
| `Enter` | ブラウザで開く |
| `Esc` | 閉じる |
| `⌘/` | このヘルプを表示/非表示 |

`⇧⌘Space` はグローバルホットキーであり `searchBox` のキーイベントとしては処理されないが、ユーザーがポップアップを開く手段そのものなのでヘルプには参考情報として掲載する。

## Risks / Trade-offs

- [Risk] ヘルプ表示中に検索デバウンス経由で `runSearch()` が解決しても、ヘルプの内容は検索結果に依存しない固定テキストなので `quickaccess-action-menu` で発生したような不整合は起きない → 追加対応不要と判断
- [Trade-off] ヘルプの内容は静的な日本語/英語の対訳表であり、実装済みショートカットが将来増えた場合(例: 今後の #57 バージョン情報UI等)は手動でヘルプ本文も更新する必要がある。自動生成の仕組みは今回のスコープでは作らない

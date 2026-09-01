## Context

現在フロントエンド（`app/dist/`）は macOS の `⌘` (Cmd / `metaKey`) を前提として実装されており、Linux 上で実行した場合でも `metaKey` (Superキー) を要求され、UI 表記も `⌘` のままとなっています。
本 change では、実行環境（macOS vs Linux）を判定し、Linux 環境での自然なキーボード操作（`Ctrl` ベース）と適切な UI 表記を実現します。

## Goals / Non-Goals

**Goals:**
- Rust バックエンドにプラットフォーム種別（`"macos"`, `"linux"` 等）を返す Tauri コマンド `get_platform` を追加する
- フロントエンド初期化時にプラットフォームを取得し、モディファイアキー判定（macOS: `metaKey`, Linux: `ctrlKey`）を切り替える
- ポップアップ内のキーバインド（コピー `KeyC`, ロック `KeyL`, ヘルプ `Slash`）を Linux 上で `Ctrl` で動作させる
- 検索ボックスでのテキスト選択中の標準コピー挙動ガード（`Ctrl+C` / `⌘C`）をプラットフォームに合わせる
- フッターヒント、アクションメニュー、ヘルプオーバーレイ内の `<kbd>` 表記を動的に切り替える

**Non-Goals:**
- グローバルホットキー登録（`hotkey.rs`）の `Shift + Ctrl + Space` 化（Issue #147 で実施）
- ユーザーによる任意のカスタムキーバインド設定機能

## Decisions

### 1. プラットフォーム判定の取得方式
- **決定**: Rust コア側に `#[tauri::command] pub fn get_platform() -> &'static str` を新設し、`initI18n()` のタイミングで `get_ui_locale` と共に取得する。
- **理由**: WebKit の `navigator.userAgent` 判定に頼るよりも、Rust 側の `cfg(target_os)` に基づく判定の方が確実で環境差異（デスクトップ環境や WebView バージョン）に強い。

### 2. キーイベント判定の抽象化
- **決定**: `app.js` に `isPrimaryMod(event)` ヘルパーを定義する：
  ```javascript
  let currentPlatform = "macos";
  function isPrimaryMod(event) {
    return currentPlatform === "macos" ? event.metaKey : event.ctrlKey;
  }
  ```
- **理由**: コピー、ロック、ヘルプ、選択テキストコピー判定の全箇所で一貫したロジックを適用できるため。

### 3. ショートカット表記の動的解決
- **決定**:
  - `shortcutHint` 表記規則:
    - macOS: `⌘C`, `⌘⇧C`, `⌥⌘C`, `⌘L`, `⌘/`, `⇧⌘Space`
    - Linux: `Ctrl+C`, `Ctrl+Shift+C`, `Ctrl+Alt+C`, `Ctrl+L`, `Ctrl+/`, `Shift+Ctrl+Space`
  - `app.js` の `buildActionsForItem`: プラットフォーム判定に基づいて適切な `shortcutHint` 文字列を生成する。
  - `i18n.js` / `app.js`: フッターヒント（`shortcutHints`）は `i18n.js` 側で `{mod}`, `{shiftMod}`, `{altMod}` 等の統一トークン置換、またはプラットフォーム別の定義により動的生成する。
  - `index.html` のヘルプオーバーレイ: 初期化時に `<kbd>` 要素内の macOS 固有記号（`⌘` → `Ctrl`, `⌥` → `Alt`, `⇧` → `Shift`）をプラットフォームに応じた表記に一括置換する。ヘルプ内の `⇧⌘Space` も Linux では `Shift+Ctrl+Space`（Issue #147 の予定挙動）に合わせる。

## Risks / Trade-offs

- **[Risk]** Linux 環境で `Ctrl+C` によるフィールドコピーを行う際、検索ボックス内のテキスト選択コピーと衝突する可能性。
  → **Mitigation**: 既存の `hasTextSelectionInSearchBox()` ガード（`selectionStart !== selectionEnd`）により、テキスト選択がある場合は `handleActionShortcut` が早期リターンして標準のクリップボードコピーが優先される。
- **[Risk]** WebKitGTK (Linux) 固有のデフォルトキーバインド（例: Ctrl+L でフォーカス移動等）との競合。
  → **Mitigation**: イベントハンドラ内で `event.preventDefault()` を適切に呼び出して WebKit 側のデフォルト動作を抑制する。実機検証で確認する。

## Why

`app/dist/style.css` は配色がすべて16進カラーコードで直書きされており、macOS のシステム外観設定(ダークモード)に連動しない。ダークモードのユーザーには白基調のウィンドウが表示され、他のネイティブアプリと並べたときの視認性・統一感を損ねる(GitHub issue #85)。また色指定が各セレクタに分散しているため、配色を見直す際の保守コストも高い。

## What Changes

- `app/dist/style.css` の `:root` に配色用の CSS カスタムプロパティ(背景・テキスト・ボーダー・アクセント・エラーなど)を定義し、既存のハードコードされたカラーコードをすべてこれらの変数参照に置き換える。
- `@media (prefers-color-scheme: dark)` ブロックで上記カスタムプロパティのダークモード用の値を定義する。
- アクセントカラーとエラーカラーは、ダークモードでは macOS 純正アプリに倣った専用値(System Blue Dark `#0a84ff` / System Red Dark `#ff453a`)を使用する。それ以外の背景・テキスト・ボーダー系の色もダークモードで視認性が保たれる値を定義する。
- `.action-menu li.focused` のようにアクセント背景の上に白背景を重ねる配色(常にアクセント色の子要素として使われる箇所)は、テーマに関わらず固定色のまま変数化する。
- スコープは CSS(`app/dist/style.css`)の変更のみとし、Tauri 側(`src-tauri`)のウィンドウ外観設定(vibrancy 等)や HTML/JS の構造変更は対象外とする。

## Capabilities

### New Capabilities
- `ui-dark-mode`: アプリの配色を CSS カスタムプロパティで定義し、macOS のシステム外観設定(ライト/ダーク)に応じて自動的に切り替える。

### Modified Capabilities
(なし)

## Impact

- 変更対象ファイル: `app/dist/style.css` のみ。
- HTML(`app/dist/index.html`)・JS(`app/dist/app.js`, `app/dist/i18n.js`)・Rust(`src-tauri`)への変更は不要。
- 既存の見た目(ライトモード)は現状のカラーコードを変数の初期値として維持するため、ライトモードでの表示に変更はない。
- 新規依存関係の追加なし。ビルド設定(`tauri.conf.json`)への影響なし。

## 1. CSS カスタムプロパティの導入

- [x] 1.1 `app/dist/style.css` の先頭に `:root` ブロックを追加し、design.md の変数マッピング表に従って `--bg-primary` / `--text-primary` / `--text-secondary` / `--border-color` / `--border-color-subtle` / `--accent-color` / `--accent-text` / `--danger-color` をライトモードの現行値で定義する
- [x] 1.2 `@media (prefers-color-scheme: dark)` 内に `:root` ブロックを追加し、design.md の変数マッピング表に従ってダークモード用の値(`--accent-text` を除く7変数)を定義する

## 2. 既存カラーコードの変数参照への置き換え

- [x] 2.1 `body`, `#master-password`, `#unlock-form button`, `#unlock-error`, `#error-screen h1`, `#error-message` のカラーコードを対応する `var(--*)` に置き換え、`grep -n '#[0-9a-fA-F]\{3,6\}'` で該当セクションに直書きの色コードが残っていないことを確認する
- [x] 2.2 `#search-box`, `#results li.focused`, `.item-icon-placeholder`, `#results li.focused .item-icon-placeholder`, `#status-footer`, `.user-avatar` のカラーコードを対応する `var(--*)` に置き換え、`grep -n '#[0-9a-fA-F]\{3,6\}'` で該当セクションに直書きの色コードが残っていないことを確認する
- [x] 2.3 `.action-menu li.focused`, `#help-overlay`, `#help-overlay dl div`, `#help-overlay dd`, `#help-overlay kbd` のカラーコードを対応する `var(--*)` に置き換え、`grep -n '#[0-9a-fA-F]\{3,6\}'` でファイル全体に `:root` ブロック以外の直書き色コードが残っていないことを確認する
- [x] 2.4 `:root` に `--field-bg` を追加し(ライト `#fff` / ダーク `#2c2c2e`)、`#master-password`, `#search-box` に `background: var(--field-bg)` と `color: var(--text-primary)` を追加する(実機確認で入力欄が UA デフォルトの白背景のまま残っていたため)

## 3. 動作確認

- [x] 3.1 `cargo build`(`app/src-tauri` 配下)を実行し、CSS 変更がビルドを壊していないことを確認する
- [x] 3.2 アプリを起動し、macOS のシステム外観設定をライト/ダークで切り替えながらアンロック画面・検索画面(通常表示・フォーカス項目・アクションメニュー)・エラー画面・ヘルプオーバーレイの配色を目視確認し、いずれのモードでもテキストが判読可能なコントラストで表示されることを確認する
- [x] 3.3 ダークモードでアンロック失敗時のエラー表示とバックエンド接続エラー画面を目視確認し、エラーテキストが強調色で判読できることを確認する

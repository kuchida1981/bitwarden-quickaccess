## Context

`app/dist/style.css` は Tauri アプリ(WKWebView ベース)の実フロントエンドで、ビルド成果物ではなく手書きのソースファイル。現状、色は全セレクタに16進カラーコードで直書きされている(`#0071e3`, `#f5f5f7`, `#1d1d1f`, `#d0d0d5`, `#e5e5ea`, `#8e8e93`, `#d70015`, `#fff` 等)。Tauri は OS の WKWebView をそのまま利用するため、CSS の `@media (prefers-color-scheme: dark)` は追加のネイティブ側コード無しで macOS のシステム外観設定を検知できる。`src-tauri` 側には現在テーマ関連のコードは存在しない。詳細は proposal.md の Why を参照。

## Goals / Non-Goals

**Goals:**
- `app/dist/style.css` の全カラーコードを `:root` で定義した CSS カスタムプロパティに置き換える
- `prefers-color-scheme: dark` でダークモード用の値を上書きし、OS のテーマ切り替えに追従させる
- ライトモードの見た目は現状から変更しない(既存カラーコードをそのまま初期値として維持)

**Non-Goals:**
- `src-tauri` 側のウィンドウ外観設定(vibrancy/透過等)の変更
- HTML 構造・JS ロジックの変更
- ライトモード/ダークモード以外のテーマ(手動切り替えUIなど)の追加

## Decisions

### 変数マッピング

以下の9つの CSS カスタムプロパティを `:root` に定義し、既存のハードコード箇所を置き換える。

| CSS変数 | ライト(現状維持) | ダーク | 用途 / 根拠 |
|---|---|---|---|
| `--bg-primary` | `#f5f5f7` | `#1e1e1e` | 全体背景、`#help-overlay` 背景 |
| `--text-primary` | `#1d1d1f` | `#f5f5f7` | 基本テキスト色 |
| `--text-secondary` | `#8e8e93` | `#98989d` | 補助テキスト(`#help-overlay dd` 等)。Apple `secondaryLabel` 相当で両モードとも視認性が高いため、ダーク値も明度を維持する近似色を採用 |
| `--border-color` | `#d0d0d5` | `#3a3a3c` | input・footer の枠線、`.item-icon-placeholder` 背景 |
| `--border-color-subtle` | `#e5e5ea` | `#2c2c2e` | ヘルプ項目の区切り線、`kbd` 背景 |
| `--accent-color` | `#0071e3` | `#0a84ff` | ボタン背景、フォーカス項目背景、アバター背景。macOS System Blue(Light/Dark)に準拠 |
| `--accent-text` | `#fff` | `#fff` | アクセント色上の文字。両モード共通で不変 |
| `--danger-color` | `#d70015` | `#ff453a` | エラーテキスト。macOS System Red(Light/Dark)に準拠 |
| `--field-bg` | `#fff` | `#2c2c2e` | `<input>` 要素(`#master-password`, `#search-box`)の背景。テキスト色は `--text-primary` を流用する |

**`--field-bg` を追加した経緯**: `<input>` 要素は border 色のみ変数化しても、背景・文字色はブラウザ(WKWebView)の UA デフォルト(常に白背景・黒文字)が適用されるため、ダークモードでも入力欄だけ白いまま取り残されることが実機確認で判明した(ライトモードでの見た目は現状の白背景のままで変化なし)。ページ背景 `--bg-primary`(ダーク `#1e1e1e`)と同色にすると入力欄の輪郭が背景に溶け込むため、macOS のテキストフィールド(NSTextField)のダーク表示に近い、ページ背景よりわずかに明るい色(`#2c2c2e`、`--border-color-subtle` と同値)を採用し、入力欄であることが視覚的にわかるようにした。テキスト色は新規変数を追加せず既存の `--text-primary` を流用する(値が完全に一致するため)。

**なぜ macOS System Blue/Red の専用ダーク値を採用するか**: ライトモードの `#0071e3` / `#d70015` をダークモードでもそのまま使い回す案も検討したが、ダーク背景上では彩度が沈んで見えづらく、ネイティブアプリとの統一感も薄れる。Apple の Human Interface Guidelines に沿った標準のダークバリアントを採用することで、追加のコントラスト調整なしに視認性と「ネイティブらしさ」を両立できる。

**`.action-menu li.focused` の扱い**: この要素は常に親(`li.focused`)のアクセント背景の上に重なる白背景ハイライトであり、周囲の色に依存しない固定配色として成立している。そのため `background:#fff; color:#0071e3` は `--accent-text` の白と `--accent-color` を使った変数参照に置き換えるが、ダークモード専用の別値は持たせない(親のアクセント背景色自体が `--accent-color` の変化に追従するため、白背景+アクセント文字の組み合わせは両モードで成立する)。

### 実装方法

```css
:root {
  --bg-primary: #f5f5f7;
  --text-primary: #1d1d1f;
  --text-secondary: #8e8e93;
  --border-color: #d0d0d5;
  --border-color-subtle: #e5e5ea;
  --accent-color: #0071e3;
  --accent-text: #fff;
  --danger-color: #d70015;
  --field-bg: #fff;
}

@media (prefers-color-scheme: dark) {
  :root {
    --bg-primary: #1e1e1e;
    --text-primary: #f5f5f7;
    --text-secondary: #98989d;
    --border-color: #3a3a3c;
    --border-color-subtle: #2c2c2e;
    --accent-color: #0a84ff;
    --danger-color: #ff453a;
    --field-bg: #2c2c2e;
  }
}

#master-password,
#search-box {
  background: var(--field-bg);
  color: var(--text-primary);
}
```

以降、各セレクタのハードコード値(`color: #1d1d1f` 等)を対応する `var(--text-primary)` 等の参照に置き換える。`rgba(255, 255, 255, 0.35)`(フォーカス項目内のプレースホルダアイコン)のような透明度付き白は `--accent-text` ベースでは表現しづらいため据え置き(アクセント背景上の半透明白として両モードで成立する)。

## Risks / Trade-offs

- [ダーク値の視認性が実機で想定と異なる可能性] → 実装後に macOS のシステム外観設定を切り替えて全画面(アンロック・検索・エラー・ヘルプ)を目視確認する(tasks.md に確認タスクを含める)。実際、初回実装では `<input>` 要素の背景・文字色が UA デフォルトのまま残り、ダークモードでも白背景になる不具合が実機確認で見つかったため `--field-bg` を追加した。
- [`prefers-color-scheme` は WKWebView が OS 設定を正しく反映することに依存] → Tauri/WKWebView は標準でこのメディアクエリをサポートしており追加設定不要。動作確認タスクで実機検証する。

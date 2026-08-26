## Context

現状の関連コードは以下のとおり。

- `app/src-tauri/src/commands.rs` の `search_items` コマンドは `Vec<VaultItemSummary>` をそのまま返す。`VaultItemSummary.login`(`LoginDetail`)には `username: Option<String>` に加え `password: Option<String>`、`totp: Option<String>`(TOTPの生シークレット)、`uris: Vec<UriEntry>` が含まれており、これらは `bw serve` の `/list/object/items` レスポンスをそのままデシリアライズしたものである。`bw` CLIの `list items` はアイテムの完全なオブジェクトを返す仕様のため、実際に平文パスワード・TOTPシークレットがフロントエンドに渡っている可能性が高い。
- `app/dist/app.js` の `renderResults()` は `item.login && item.login.username` でユーザー名を読んでいる。`searchBox` の `keydown` ハンドラ(`handleActionShortcut`)が `⌘C` 系のショートカットを直接処理しており、メニューのような中間UIは存在しない。
- `app/dist/style.css` には `#results li .hints`(フォーカス時のみ表示されるショートカットヒント)のスタイルが既にある。

## Goals / Non-Goals

**Goals:**
- 検索結果としてフロントエンドに渡るデータから、パスワード・TOTPの実値を排除し、有無を示す真偽値に置き換える
- アイテムにフォーカスした状態で → キーを押すと、そのアイテムが実際に持つフィールドに応じたアクション一覧をその場に展開し、↑/↓ + Enter またはクリックで実行できるようにする
- 既存のダイレクトショートカット(`⌘C` 等)の挙動・spec要件は変更しない

**Non-Goals:**
- 1Password本家にあるがこのアプリにまだ実装されていない機能(自動入力、新規ウィンドウで開く等)に対応するメニュー項目の追加
- アクションメニューの見た目のうち、フォント・配色等の全面的なデザイン刷新(既存の `.hints` 相当のスタイルを踏襲する)
- `bw serve` 側のAPI仕様変更(サーバー側でフィールドを削って返すようにする、等)。あくまでクライアント(Tauriアプリ)側で受け取った後にフロントエンドへ渡す情報を絞り込む

## Decisions

### 1. `search_items` の戻り値を `SearchResultItem` DTOに変更する

`app/src-tauri/src/commands.rs` に以下を追加する。

```rust
#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchResultItem {
    pub id: String,
    pub name: String,
    pub username: Option<String>,
    pub has_password: bool,
    pub has_totp: bool,
    pub has_url: bool,
}

impl From<VaultItemSummary> for SearchResultItem {
    fn from(item: VaultItemSummary) -> Self {
        let username = item.login.as_ref().and_then(|l| l.username.clone());
        let has_password = item.login.as_ref().and_then(|l| l.password.as_ref()).is_some();
        let has_totp = item.login.as_ref().and_then(|l| l.totp.as_ref()).is_some();
        let has_url = item
            .login
            .as_ref()
            .map(|l| l.uris.iter().any(|u| u.uri.is_some()))
            .unwrap_or(false);

        Self {
            id: item.id,
            name: item.name,
            username,
            has_password,
            has_totp,
            has_url,
        }
    }
}
```

`search_items` コマンドの戻り値型を `Result<Vec<SearchResultItem>, String>` に変更し、`client.search_items(&query).await?` の結果を `.into_iter().map(SearchResultItem::from).collect()` してから返す。`VaultItemSummary` / `LoginDetail` 自体(バックエンド内部でのデシリアライズ用の型、`copy_field` / `open_in_browser` が引き続き実値を必要とする)は変更しない。

**代替案: `VaultItemSummary` に `#[serde(skip)]` 等でフロントエンド提供時だけpassword/totpを隠す** — 却下。`VaultItemSummary` は `bw serve` からのデシリアライズにも使う共有の型であり、シリアライズ方向にだけ条件を付ける(diff serialize/deserialize)のは複雑で分かりにくい。IPC境界(`commands.rs`)で明示的に別のDTOへ変換する方が、「フロントエンドに渡してよい情報はこれだけ」という境界が型として一目で分かる。

### 2. アクションメニューは検索結果一覧の描画状態(JS内の変数)として持ち、DOMフォーカスは移動させない

現状、キーボード入力は常に `searchBox`(`<input>`要素)がDOMフォーカスを持ったまま、`keydown` イベントで一覧の見た目上のフォーカス(`focusedIndex`)を操作している。アクションメニューの展開状態もこれに倣い、新しいJS変数(例: `let actionMenuOpen = false;` と `let actionMenuFocusIndex = -1;`)で管理し、DOM要素へのフォーカス移動は行わない。`searchBox` の `keydown` ハンドラ内で `actionMenuOpen` の値に応じて処理を分岐する。

**代替案: メニュー項目を実際にfocusableなDOM要素にしてTab/矢印キーでネイティブフォーカス移動する** — 却下。検索ボックスからフォーカスを外すと、続けて文字入力したときに検索が継続できなくなる、フォーカスロストで自動的にポップアップが閉じる(`popup.rs` の `Focused(false)` ハンドラ)といった既存の挙動と衝突するリスクが大きい。

### 3. アクションメニューのキー操作

`actionMenuOpen === false`(通常状態)のとき:
- `ArrowRight`: フォーカス中のアイテムが1つ以上の実行可能アクションを持つ場合、そのアイテム用のアクション一覧を構築して `actionMenuOpen = true`、`actionMenuFocusIndex = 0` にし、再描画する。実行可能アクションが1つもない場合は何もしない

`actionMenuOpen === true` のとき:
- `ArrowDown` / `ArrowUp`: `actionMenuFocusIndex` をアクション一覧の範囲内で移動する(一覧のアイテム間移動 `moveFocus` とは独立)
- `Enter`: `actionMenuFocusIndex` が指すアクションを実行する(実行後の挙動は既存の `runAction` と同じ: フィードバック表示→ポップアップを閉じる)
- `ArrowLeft` または `Escape`: `actionMenuOpen = false` にして通常の一覧表示に戻る(Escapeキー自体でのポップアップ全体クローズは別issue #54のスコープであり、本changeでは「メニューを閉じて一覧に戻る」までを扱う)
- 上記以外のキー(検索文字入力等)は無視する(メニュー表示中に検索語を変更されると表示中のメニューの前提が崩れるため)

既存の直接ショートカット(`⌘C` 等、`handleActionShortcut`)は `actionMenuOpen` の値によらず従来通り動作する(メニューを閉じている・開いているに関わらず、いつでもダイレクトショートカットで実行できる)。

### 4. アクション一覧の構築とフィールド出し分け

対象アイテム(`SearchResultItem`)から、以下の順序で実行可能なアクションの配列を構築する関数を新設する(例: `buildActionsForItem(item)`)。

```
[
  { key: "username", label: t("actionCopyUsername"), shortcutHint: "⌘C",  enabled: !!item.username },
  { key: "password", label: t("actionCopyPassword"), shortcutHint: "⌘⇧C", enabled: item.has_password },
  { key: "totp",     label: t("actionCopyTotp"),      shortcutHint: "⌥⌘C", enabled: item.has_totp },
  { key: "browser",  label: t("actionOpenBrowser"),   shortcutHint: "Enter", enabled: item.has_url },
]
```
`enabled: false` の項目はメニューの配列から除外して描画する(要件通り、フィールドが無い項目はメニューに一切表示しない)。

各アクションの実行本体(`runAction(...)` を呼ぶ処理)は、既存の `handleActionShortcut` 内で `⌘C` 等のショートカット押下時に呼んでいるものと同じ関数を共有し、二重実装しない(`key` に応じて `copy_field` / `open_in_browser` の呼び出しに振り分ける小さなディスパッチ関数を新設して両方から呼ぶ)。

### 5. UI表現: フォーカス中の `<li>` の中に入れ子の `<ul class="action-menu">` を描画する

既存の `#results li .hints`(フォーカス時のみ表示するショートカットヒント)と同様に、`renderResults()` の中でフォーカス中の行にのみアクションメニューの `<ul>` を追加描画する。`actionMenuOpen` が `true` の間は `.hints` の代わりに `.action-menu` を表示する(同時に両方は出さない)。

## Risks / Trade-offs

- [Risk] `search_items` の戻り値の形が変わることで、既存の `app.js` の `item.login.username` 参照箇所を直し忘れると、ユーザー名表示が壊れる(`undefined` になる) → [Mitigation] タスクを「型変更」と「参照箇所修正」をセットで1つのタスクにまとめ、修正漏れを実機確認で検出する
- [Risk] メニュー表示中に検索デバウンス(`SEARCH_DEBOUNCE_MS`)経由で `renderResults()` が呼ばれ、メニューの前提(対象アイテム)が変わってしまう可能性 → [Mitigation] 決定3の通り、メニュー表示中は検索文字入力自体を無視する(または `actionMenuOpen` を `false` に戻してから通常の検索処理に委ねる)。実装時にどちらが自然か判断する
- [Trade-off] メニューの状態管理をDOMフォーカスに頼らずJS変数で行うため、スクリーンリーダー等の支援技術からは「メニューが開いている」ことが伝わりにくい可能性がある。既存の実装もアクセシビリティ対応を特にしていないため、本changeのスコープでは対応しない

## Context

ポップアップウィンドウ(`popup::POPUP_LABEL`)は非表示時も`window.hide()`のみで破棄されない(`commands::hide_popup`)。そのためフロントエンド(`app/dist/app.js`)のJS変数・DOM状態は非表示中もメモリ上に保持されている。現状は再表示時のハンドラ(`handleShown` / `syncScreenWithBackend`)が`actualScreen === "search"`のケースで無条件に`searchBox.value = ""`をセットし、`runSearch("")`を呼んでいるため、既に保持されているはずの状態を毎回自前で捨てている。

非表示のトリガーは3種類ある(いずれも`hide_popup`または`WindowEvent::Focused(false)`経由でRust側`window.hide()`に到達する):
- フォーカスロス(`popup.rs:42-43`、他アプリクリック等)
- Escapeキー / アクション成功時のコピー等(`app.js`から`invoke("hide_popup")`)
- ⌘Lによる明示的ロック(`performLock` → `invoke("lock")` → `syncScreenWithBackend()`で画面遷移。ウィンドウ自体は非表示にならないこともあるが、`lockState`が変わるため`actualScreen`が`"search"`から外れる)

## Goals / Non-Goals

**Goals:**
- ポップアップが非表示になってから30秒以内に再表示された場合、検索ボックスの文字列と検索結果一覧を保持する
- 保持された状態で再表示された際、検索ボックスのテキストを全選択状態にする
- 30秒を超えた場合、⌘Lロック時、バックエンド切断時は従来通り検索状態をクリアする

**Non-Goals:**
- Rust側でのウィンドウ非表示時刻の記録・バックエンドでの永続化(フロントエンドのJS変数のみで完結させる)
- アプリケーション再起動をまたいだ状態保持
- TTLをユーザー設定で変更可能にすること(固定値30秒とする)

## Decisions

### 非表示時刻の記録場所: フロントエンドJS変数
`popup.rs`の`WindowEvent::Focused(false)`ハンドラや`hide_popup`コマンドにRust側でタイムスタンプを持たせる案もあったが、フロントエンドは既に`POPUP_SHOWN_EVENT`(表示時)を受け取っている一方、非表示イベントの通知はRust→フロントエンドに存在しない。新規にIPCイベントを追加するより、フロントエンド側で「非表示につながるトリガー(Escape実行、アクション成功、blur)が発生したタイミング」を捉えて`hiddenAt`変数に`Date.now()`を記録する方がAPI追加なしで完結し、既存パターン(状態はフロントJS変数で管理)にも合致する。
ただし`WindowEvent::Focused(false)`によるフォーカスロス隠しはRustからしかトリガーされないため、フロントエンド側で捕捉できない。この場合はDOM側の`window`の`blur`イベント(ネイティブブラウザイベント、Tauriのフォーカス変化と連動)をリッスンして同様に`hiddenAt`を記録する。

### TTL判定のタイミング: `handleShown`内で一括判定
`syncScreenWithBackend`と`handleShown`の両方に「検索画面初期化」の分岐があるため、TTL判定ロジックを共通関数化し(例: `shouldRetainSearchState()`)、両箇所から呼び出す。既存の`actualScreen === lastKnownScreen` / `actualScreen !== lastKnownScreen`の分岐構造は変えず、内側の「クリアするかどうか」だけをTTL判定に置き換える。

### 全選択のタイミング
状態を保持したまま`searchBox.focus()`した直後に`searchBox.select()`を呼ぶ。`runSearch()`は呼ばない(既存の検索結果DOMをそのまま使う)。

## Risks / Trade-offs

- [リスク] `blur`イベントとRust側`hide_popup`呼び出しのタイミングがずれる可能性(例: Escapeキー押下時は`hide_popup`呼び出しの方が先) → `hiddenAt`はどちらのトリガーでも同じ関数から記録するため、経路によらず一貫した記録になる。多少の記録タイミングのズレ(数十ms)はTTL判定(30秒)に対して無視できる。
- [リスク] TTL以内での再表示時に検索結果一覧がバックエンド側の最新状態と乖離する可能性(保持中にvaultの中身が変わった場合) → 本changeのスコープ外とする。1Password Quick Accessも同様の挙動であり、次回入力や30秒経過で解消される軽微な問題として許容する。
- [トレードオフ] TTLを固定値30秒にすることで設定の柔軟性は失うが、issueの要求(1Password相当のUX)とスコープの単純さを優先する。

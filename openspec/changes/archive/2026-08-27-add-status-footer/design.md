## Context

現在の実装:
- ショートカットヒントは `app/dist/app.js` の `buildTrailingBlock`(`add-item-icons` change で `renderResults` から分離済み)が、フォーカス中の行にのみ `.hints` div として埋め込んでいる。長い文言(例: `⌘C ユーザー名 / ⌘⇧C パスワード / ⌥⌘C TOTP / Enter ブラウザで開く / → メニュー`)が折り返され、一覧の見た目を圧迫している(Issue #64)。
- `bw serve` の `/status` レスポンスには `userEmail` / `userId` が含まれる(`http_client.rs` のテスト用モックJSON参照)が、`StatusTemplate` は `status: String` のみを持ち、パースされずに破棄されている(Issue #70)。
- ポップアップウィンドウは `app/src-tauri/src/popup.rs` の `WIDTH = 420.0` / `HEIGHT = 480.0` で固定サイズ。

ユーザーからの参考情報(1Passwordクイックアクセスのスクリーンショット)により、実際の1Password UIでは「ログイン中アカウント」表示はフッタには無く、フッタはショートカットヒントのみであることが判明した。ただし本プロジェクトでは、探索セッションでの合意により、ヒント(Issue #64)とアカウント表示(Issue #70)を同じフッタ領域にまとめて表示する方針を維持する(1Passwordの厳密な再現ではなく、同種の「グローバルな固定領域」という考え方を踏襲する)。

## Goals / Non-Goals

**Goals:**
- ショートカットヒントを行内から、検索画面下部の常時表示フッタへ移動する。
- ログイン中アカウントを(頭文字アバター+hoverでのメールアドレス表示という形で)フッタから確認できるようにする。
- フッタを1行に収めるため、ポップアップウィンドウの幅を広げる。

**Non-Goals:**
- 1Passwordのレイアウトの厳密な再現(行にドメイン/URLを追加表示する等)は行わない。
- 複数Bitwardenアカウントの切り替えUIは扱わない(本アプリは単一の `bw` CLIセッションを前提とする既存設計を変えない)。
- アンロック画面(ロック中)へのアカウント表示追加は、Issue #70の主要な要求(検索画面での表示)のスコープ外とする。

## Decisions

### フッタのレイアウト
検索画面(`#search-screen`)の `#results` / `#empty-message` の下に、常時表示の `#status-footer` を新設する。横並びで左にログイン中アカウントの頭文字アバター(`#current-user-avatar`)、右にショートカットヒント(`#footer-hints`、既存の `shortcutHints` i18n文言をそのまま使う、フォーカス中アイテムによらず固定)を配置する。

### アカウント表示: 頭文字アバター + hoverでメールアドレス表示
「どのアカウントでログイン中か知りたい」という要求に対し、メールアドレスをフッタに常時テキスト表示する案と、頭文字アバター(小さな丸に先頭1文字)+ hoverで完全なメールアドレスをネイティブtooltip(`title`属性)表示する案を比較検討した。省スペースで一覧の視認性を損なわないことを優先し、後者を採用する。
- `#current-user-avatar` は `<span>` 要素とし、`textContent` にメールアドレスの先頭1文字(大文字化)を設定、`title` 属性に完全なメールアドレスを設定する。
- メールアドレスが取得できない(`None`)場合は、アバター自体を非表示にする(`display: none`)。
- 完全なメールアドレスを常時見たいという要求ではなく「今どのアカウントか確認したいときに確認できればよい」という要求のため、hoverでの確認で十分と判断する。将来的に不十分と分かった場合は、テキスト表示への変更を再検討する。

### ウィンドウ幅の拡大
`app/src-tauri/src/popup.rs` の `WIDTH` を `420.0` から `520.0` に拡大する(1Passwordクイックアクセスの横幅を参考にした概算値。実機で1行に収まるか確認し、必要なら実装時に微調整する)。`HEIGHT` は変更しない(フッタの高さ分は `#results` の `flex: 1` が吸収する)。

### 行内ヒントの削除方法
`buildTrailingBlock` の非アクションメニュー分岐(現状 `.hints` divを返している部分)を、空の `<span class="row-trailing-placeholder">` に置き換える。これにより:
- 視覚的にはヒントが行から消える(Issue #64の要求を満たす)。
- `updateFocusRows` / `refreshFocusedRowTrailing`(`add-item-icons` changeで導入)が前提とする「各行の最後の子要素は常にトレイリングブロックである」という不変条件を壊さずに済む(アクションメニュー展開時は引き続き同じ位置に挿入される)。

### アカウント情報の取得経路
- `StatusTemplate` に `user_email: Option<String>` を追加してパースする。
- `BwServeClient::status()` の戻り値を `LockStatus` 単体から `StatusInfo { lock_status: LockStatus, user_email: Option<String> }` に変更する(呼び出し元は `main.rs::sync_initial_status` の1箇所のみで、破壊的変更の影響範囲は小さい)。
- `AppState` に `user_email: Option<String>` を保持するフィールドとアクセサ(`set_user_email` / `user_email()`)を、既存の `last_error` / `port` と同じパターンで追加する。`sync_initial_status` がLocked/Unlocked判定時にあわせて設定する。
- 新規Tauriコマンド `get_current_user() -> Option<String>` を追加し、`state.user_email()` を返す。
- フロントエンドは検索画面表示時(`handleShown`/`syncScreenWithBackend` で検索画面に遷移する際)に `invoke("get_current_user")` を呼び、`#current-user-avatar` の `textContent`(先頭1文字・大文字化)と `title`(完全なメールアドレス)に反映する。`None` の場合はアバター要素を非表示にする。

### アカウント情報の更新頻度
検索画面はアンロック済みの間のみ表示され、`bw` CLIのセッション中にログインアカウントが変わることは想定しない(既存設計を踏襲)。そのため `user_email` は起動時の `sync_initial_status` で一度取得すればよく、アンロックのたびに再取得する必要はない。

## Risks / Trade-offs

- [ウィンドウ幅を広げることで、既存ユーザーの画面レイアウトの見た目が変わる] → 数値は実装時に実機で最終調整する。大幅な変更ではないため、影響は軽微と判断する。
- [`status()` の戻り値変更が破壊的] → 呼び出し元は `main.rs` の1箇所のみで、`cargo build` によるコンパイルエラーで漏れなく検出できる。
- [`userEmail` がbw serveのレスポンスに含まれないケース(将来のbw CLIバージョン変更等)] → `Option<String>` として扱い、`None` の場合はフッタのアカウント表示欄を空にするだけで、他の機能に影響しない。

## Open Questions

- フッタを1行に収めるための正確なウィンドウ幅は実機確認で微調整する(520pxは暫定値)。

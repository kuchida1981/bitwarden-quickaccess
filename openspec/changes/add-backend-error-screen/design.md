## Context

`AppState`(`app/src-tauri/src/backend/state.rs`)は既に `last_error()` アクセサを持っており、`start_backend()`/`sync_initial_status()`(`main.rs`)がpreflight失敗・`bw serve`起動失敗・未ログイン等の際に `set_error()` でメッセージを記録している。しかし、この値をフロントエンドへ渡す経路が存在しない。

フロントエンドの `handleShown()`(`app.js`)は `get_lock_state` の戻り値(`"disconnected"` | `"locked"` | `"unlocked"`)のうち `"unlocked"` 以外をすべて `"unlock"` 画面(マスターパスワード入力フォーム)にマッピングしており、`"disconnected"` の場合に不適切なUIを表示してしまう。

## Goals / Non-Goals

**Goals:**
- `disconnected` 状態のとき、原因を示す専用画面を表示する。
- `bw`未検出・`bw serve`未対応・未ログインのいずれのケースも同じ仕組み(`last_error()`の文字列をそのまま表示)でカバーする。
- 既存の `locked`/`unlocked` 画面遷移ロジックには影響を与えない。

**Non-Goals:**
- preflightエラーメッセージ自体の英語ローカライズ(現状Rust側のエラー文言は日本語固定であり、`unlockError` 表示等の既存箇所も同様に未対応。本changeのスコープ外とする)。
- 接続の自動リトライ・再接続UI(「今すぐ再試行」ボタン等)の追加。ユーザーは原因を解消してアプリを再起動する想定(現状の起動失敗時の運用と同じ)。
- トレイメニューへの詳細エラーメッセージ表示(現状もアイコン・ステータス文言で`disconnected`状態自体は表現されているため、詳細メッセージの追加表示は本changeでは扱わない)。

## Decisions

- **`get_lock_state` の戻り値は変更せず、新規コマンド `get_backend_error` を追加する**。既存コマンドの戻り値型を変更する破壊的な選択肢もあったが、呼び出し元(`handleShown()`)を含め影響範囲を最小化するため、`disconnected` のときだけ追加で問い合わせる設計にする。
  - シグネチャ: `fn get_backend_error(state: tauri::State<'_, AppState>) -> Option<String>`
- **エラーメッセージの文言はそのまま(未加工)で表示する**。既存の `unlockError`(アンロック失敗時)も同様にRust側の生の文字列をそのまま表示しており、一貫した方式とする。
- **`handleShown()` の画面判定を3値のswitch的分岐にする**: `"unlocked"` → `"search"`、`"locked"` → `"unlock"`、それ以外(`"disconnected"`)→ `"error"`。

## Risks / Trade-offs

- [エラーメッセージが日本語固定のため、英語環境のユーザーにも日本語のエラー文言が表示される] → 既存の `unlockError` 表示と同じ制約であり、本changeで新たに悪化させるものではない。将来的な改善課題として残す。
- [`disconnected` 状態から復帰(例: ユーザーが `bw` をインストールしてアプリを再起動)する際の動線が「アプリを再起動する」以外に無い] → 現状の起動失敗時の運用を変えるものではなく、本changeのスコープ外とする。

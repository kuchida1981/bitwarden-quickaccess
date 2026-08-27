## Context

`app/src-tauri/src/tray.rs` の `setup_tray` が構築するメニューは、ステータス表示・ホットキー登録状況・自動起動トグル・「今すぐロック」・About・GitHubリンク・終了のみで構成されており、クイックアクセスポップアップ自体を開く項目が無い。ポップアップの表示/非表示切り替えは `popup::toggle_popup(app: &AppHandle)`(`app/src-tauri/src/popup.rs`)が既に提供しており、現状はグローバルホットキーのコールバックからのみ呼ばれている。

## Goals / Non-Goals

**Goals:**
- トレイメニューからクイックアクセスの表示/非表示を切り替えられるようにする。
- ホットキー(⇧⌘Space)の存在にメニュー経由で気づけるようにする。

**Non-Goals:**
- ホットキー登録に失敗している場合の代替導線としての詳細な案内文言の追加(既存の `hotkey_unregistered_prefix` 表示で足りるとし、本changeでは新規メニュー項目のラベルにホットキーを併記するのみに留める)。
- メニュー項目のアイコン装飾等、見た目の作り込み。

## Decisions

- 新規メニュー項目 `OPEN_QUICKACCESS_ITEM_ID` を、`hotkey_item`(ホットキー登録状況表示)の直後・最初の区切り線の前に配置する(Issueの「ステータス表示の下あたり」という要望に沿う)。
- ラベルは `m.open_quickaccess_label`(例: 「クイックアクセスを開く (⇧⌘Space)」)とし、既存の `hotkey_registered`/`hotkey_unregistered_prefix` と同じ `i18n::Messages` 構造体に追加する。
- クリック時は `on_menu_event` に `OPEN_QUICKACCESS_ITEM_ID => crate::popup::toggle_popup(app)` を追加する。他の項目(`LOCK_ITEM_ID` 等)と同じ即時実行パターンで、非同期処理は不要(`toggle_popup` は同期関数)。

## Risks / Trade-offs

- [メニュー項目が増えることで既存メニューが縦に長くなる] → 1項目のみの追加であり、影響は軽微と判断する。

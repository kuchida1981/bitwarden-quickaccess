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

## 【実機確認で発覚した重複表示の解消】ホットキー登録成功時の状態表示を削除

実装後の実機確認で、ホットキー登録が成功している場合、既存の「ホットキー: ⇧⌘Space」表示(`hotkey_item`)と新規の「クイックアクセスを開く (⇧⌘Space)」表示が、同じ情報(ホットキーの組み合わせ)を重複して表示してしまうことが判明した。

**対応**: `hotkey_item` は、ホットキー登録に**失敗**した場合(`hotkey_warning: Some(reason)`)のみメニューに含める。失敗時の警告表示(`⚠ ホットキー未登録: {reason}`)は、新規項目のラベルだけでは伝わらない重要な診断情報であり、引き続き表示する。登録成功時(`hotkey_warning: None`)は `hotkey_item` 自体をメニューから除外し、ホットキーの存在は新規の「クイックアクセスを開く (⇧⌘Space)」項目のラベルのみで示す。

これに伴い、成功時専用だった `i18n::Messages::hotkey_registered` フィールドは参照されなくなるため削除する(未使用コードとして `cargo clippy -D warnings` で検出されるため)。

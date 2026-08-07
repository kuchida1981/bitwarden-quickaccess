## Context

`lib/search.sh`(検索画面)と `lib/fields.sh`(フィールド選択画面)は、fzf の `--bind` でコピー用ショートカット(`ctrl-u`=ユーザー名、`ctrl-p`=パスワード、`ctrl-t`=TOTP)を割り当てている。`man fzf` の KEY/EVENT BINDINGS を確認したところ、`ctrl-p`(`up-match`)と `ctrl-u`(`unix-line-discard`)は fzf のデフォルトバインドと一致しており、`--bind` による再バインドはデフォルト動作を完全に上書きする(chain して明示的に繋がない限り)。`ctrl-t` はデフォルトバインドが存在しないため無事だった。

issue #13 でこの問題を報告し、探索の結果、以下の方針を決定した:

- 修正スコープ: `ctrl-p` だけでなく `ctrl-u` も一緒に見直す(同一の欠陥クラスのため)
- 置き換え方式: `alt-p` / `alt-u` へ変更し、`p`=password / `u`=username のニーモニックを維持する

## Goals / Non-Goals

**Goals:**
- `ctrl-p`/`ctrl-n`(選択移動)と `ctrl-u`(クエリ編集)を fzf のネイティブ動作に戻す
- コピー用ショートカットのニーモニック(p/u/t)を維持したまま、fzf デフォルトと衝突しないキーに移す
- 検索画面・フィールド選択画面で一貫したキー割り当てを保つ(既存要件どおり)

**Non-Goals:**
- TOTP のキー割り当て(`ctrl-t`)の変更(衝突がないため対象外)
- キーバインドをユーザー設定可能にする仕組みの導入(将来的な拡張候補だが本 change のスコープ外)
- fzf 以外のツール連携(clipboard/keychain 等)の変更

## Decisions

### 決定1: 置き換え先キーは `alt-p` / `alt-u`

**選択肢と比較:**

| 案 | 内容 | 長所 | 短所 |
|---|---|---|---|
| A. `alt-p`/`alt-u`(採用) | Alt 修飾でニーモニックを維持 | p=password, u=username の覚えやすさを維持。fzf デフォルト未使用 | ターミナルの Meta キー送信設定に依存する場合がある(下記リスク参照) |
| B. `ctrl-o`/`ctrl-r` 等、fzf 未使用の Ctrl キー | ニーモニックを捨てて安全な Ctrl キーに割り当て | ターミナル設定に依存せず確実に動く | o/r と password/username の対応が直感的でない。ユーザーが覚え直す必要がある |
| C. `ctrl-p:up-match+execute-silent(...)` のように既存キーに追記(chain) | デフォルト動作を残しつつコピーも実行 | キー変更不要 | 1キーに2つの意味(移動 or コピー)を持たせることになり、`{1}`(現在ハイライト行)を使うコピーと「移動した後の行」が曖昧になる。ユーザーの意図(移動したいのかコピーしたいのか)を区別できない |

案 A を採用。ユーザーとの探索で、ニーモニックの維持を優先する判断となった。

### 決定2: `ctrl-t` は変更しない

`ctrl-t` は fzf にデフォルトバインドが存在せず、衝突が起きていないため、一貫性のためだけに変更する必要はないと判断。

## Risks / Trade-offs

- [Risk] Alt キーの送信はターミナルエミュレータの設定に依存する(例: macOS 標準 Terminal.app は Profile > Keyboard で「Use Option as Meta Key」を有効化しないと `alt-p`相当のエスケープシーケンスが送られない場合がある)→ Mitigation: README に注記を追加し、動作しない場合の対処(設定変更 or 別ターミナル利用)を案内する
- [Risk] 既存ユーザーは `ctrl-p`/`ctrl-u` に慣れている(**BREAKING**)→ Mitigation: README とヘッダー表示(`--header`)の両方で新キーを明示し、画面上で常に確認できるようにする
- [Trade-off] `ctrl-o`/`ctrl-r` 案(B)より覚えやすいが、ターミナル依存というリスクを受け入れる判断をしている

## Migration Plan

- 破壊的変更だが、設定ファイルやマイグレーションスクリプトは不要(fzf の `--bind` 引数を変更するのみ)
- リリースノート相当として README の更新で周知する

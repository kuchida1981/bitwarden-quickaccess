## Context

`fix-copy-keybind-conflict`(アーカイブ済み)で、`ctrl-u`/`ctrl-p` が fzf のデフォルトバインド(`unix-line-discard`/`up-match`)と衝突する問題を、`alt-u`/`alt-p` への変更で解消した。その design.md では「Ctrl キー案(`ctrl-o`/`ctrl-r` 等)はニーモニックが直感的でない」という理由で見送り、ニーモニック維持を優先して Alt 案を採用していた。

Issue #19 で、その `alt-u`/`alt-p` が macOS + Alacritty の組み合わせで実際に動作しないことが判明した。調査の結果:

- `alt-*` は端末が Option キー押下を ESC プレフィックス付きシーケンスとして送信する設定になっていないと fzf 側で検知できない。この設定(Terminal.app の「Use Option as Meta Key」、iTerm2 の「Esc+」、Alacritty の `option_as_alt` 等)はターミナルごとに異なり、デフォルトでは無効なことが多い
- 代替として検討した `ctrl-enter`/`shift-enter` は、fzf の man に記載されたキー語彙に存在しない(fzf は Enter を `ctrl-m` として単一視しており、修飾キー付き Enter を区別する仕組みへの言及が一切ない)
- 代替として検討した F-Key(f1-f12)も、macOS の MacBook キーボードでは輝度・Mission Control 等のメディア機能がデフォルト割り当てされており、素のファンクションキーコードを送るには `fn` 同時押しかシステム設定変更が必要で、`alt-*` と同種の環境依存問題を OS 層で抱える

「ターミナル・OS の設定に依存しない実装にする」という要求を満たせるのは、C0 制御コードとして全端末・全 OS で一貫して送信される `ctrl-[a-z]` のうち、fzf のデフォルト割り当てがなく `ctrl-p`/`ctrl-n`/`ctrl-u` を侵さないキーのみ、という結論に至った。つまり前回見送った「Ctrl キー案」を、ニーモニックより環境非依存を優先する形で採用し直す。

## Goals / Non-Goals

**Goals:**
- ユーザー名/パスワードの直接コピーのキーバインドを、ターミナル・OS の設定に一切依存しないキーへ変更する
- `ctrl-p`/`ctrl-n`(選択移動)・`ctrl-u`(クエリ編集)という fzf のネイティブ動作は今回も維持する(`fix-copy-keybind-conflict` からの継続方針)
- 検索画面・フィールド選択画面で一貫したキー割り当てを保つ(既存要件どおり)

**Non-Goals:**
- TOTP のキー割り当て(`ctrl-t`)の変更(環境依存の問題が起きていないため対象外)
- キーバインドをユーザー設定可能にする仕組みの導入(将来的な拡張候補だが本 change のスコープ外)
- fzf 以外のツール連携(clipboard/keychain 等)の変更

## Decisions

### 決定1: 置き換え先キーは `ctrl-o`(ユーザー名)/ `ctrl-r`(パスワード)

**選択肢と比較:**

| 案 | 内容 | 長所 | 短所 |
|---|---|---|---|
| A. `alt-u`/`alt-p`(現状) | Alt 修飾でニーモニックを維持 | p=password, u=username の覚えやすさ | ターミナルの Meta キー送信設定に依存し、Alacritty 等で実際に機能しない(Issue #19 で確認済み) |
| B. `ctrl-enter`/`shift-enter` | 修飾キー付き Enter | 概念上シンプル | fzf のキー語彙に存在せず、そもそもバインド不可能 |
| C. F-Key(f2/f3 等) | ファンクションキーへ変更 | ターミナル設定には非依存 | macOS の MacBook キーボードは F-Key にメディア機能をデフォルト割り当てしており、`fn` 同時押し等が必要になる別種の環境依存を抱える |
| D. `ctrl-o`/`ctrl-r`(採用) | fzf 未使用の Ctrl キーへ変更 | C0 制御コードのため端末・OS 設定に一切依存せず確実に動作する。`fix-copy-keybind-conflict` の時点で既に検討済みの案 | ニーモニックを失う(o/r と username/password の対応が直感的でない) |

案 D を採用。`ctrl-s`(XON/XOFF の歴史的連想)・`ctrl-x`(「切り取り」の連想)・`ctrl-z`(「元に戻す/一時停止」の連想)は他機能との誤認を避けるため候補から除外し、比較的連想の少ない `ctrl-o`/`ctrl-r` を選定した。ニーモニック喪失は、`--header` に常時キー説明を表示することで軽減する(`ctrl-t` がニーモニックと空きキーの偶然の一致に過ぎないにもかかわらず、これまで運用上の問題が出ていない実績を踏まえた判断)。

### 決定2: `ctrl-t` は変更しない

`ctrl-t` は fzf にデフォルトバインドが存在せず、Alt キーのような環境依存の問題も起きていない。一貫性のためだけに変更する必要はないと判断。

## Risks / Trade-offs

- [Risk] 既存ユーザーは `alt-u`/`alt-p`(またはそれ以前の `ctrl-u`/`ctrl-p`)に慣れている可能性がある(**BREAKING**、2回目のキー変更)→ Mitigation: README とヘッダー表示(`--header`)の両方で新キーを明示し、画面上で常に確認できるようにする
- [Trade-off] ニーモニックを失うため、初回利用時の学習コストが `alt-u`/`alt-p` よりわずかに高い → Mitigation: ヘッダー常時表示、README での明記
- [Risk] 将来 fzf が `ctrl-o`/`ctrl-r` にデフォルトバインドを追加した場合、再度衝突が起きうる → Mitigation: 現時点では両キーとも fzf 本体の man に記載がないことを確認済み。将来のバージョンアップ時に man を再確認する運用でカバー(自動検知の仕組みは本 change のスコープ外)

## Migration Plan

- 破壊的変更だが、設定ファイルやマイグレーションスクリプトは不要(fzf の `--bind` 引数を変更するのみ)
- リリースノート相当として README の更新で周知する

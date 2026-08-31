## Context

`app/src-tauri` に `rustfmt.toml` は存在せず、rustfmt はデフォルト設定(edition 2021 のデフォルトスタイル)で動作する。現状 `cargo fmt --check` を実行すると 12 ファイル・53 箇所で差分が検出される(いずれも折り返しや空行など整形上の差分で、ロジック変更を伴うものはない)。CI ジョブ (`ci.yml`) は `defaults.run.working-directory: app/src-tauri` を設定済みで、既存の `cargo build` / `cargo test` / `cargo clippy` ステップはこれに従っている。

## Goals / Non-Goals

**Goals:**
- 未フォーマットのコードが `main` にマージされるのを CI で機械的に防ぐ
- 既存コードを一括フォーマットし、`cargo fmt --check` 追加時点で CI がグリーンな状態にする
- フォーマット差分が純粋な整形のみであり、ビルド・テスト・clippy の結果に影響しないことを保証する

**Non-Goals:**
- `rustfmt.toml` を新規作成してプロジェクト固有のスタイルルールを定義すること(デフォルト設定をそのまま使う)
- CI のジョブ構成自体(マトリクス化、OS追加など)を見直すこと
- pre-commit フックへの `cargo fmt` 導入(CLAUDE.md に記載の通り pre-commit は現時点で未設定であり、本changeのスコープ外)

## Decisions

**1. `cargo fmt` の一括適用と CI ステップ追加を同じ change 内で行う**
issue の acceptance criteria が「既存コードがすべて fmt 済みであることを確認する」を含むため。ステップだけ先に追加すると既存コードが原因で CI が即座に赤くなり、実質的に別途 fmt PR が必須になる。両方を1つの change にまとめることで、マージ後すぐにグリーンな CI を維持できる。

**2. コミットを「既存コードの一括フォーマット」と「CI ステップ追加」の2つに分ける**
1コミットにまとめると diff が「整形変更」と「CI設定変更」で性質が異なり、レビューやリバートがしづらくなる。2コミットに分けることで、フォーマット変更のみを機械的差分として確認しやすくする。

**3. `working-directory` はジョブの `defaults.run.working-directory` に委譲する**
issue の実装ノートでは `cargo fmt --check` ステップに個別の `working-directory: app/src-tauri` を明示しているが、既存の3ステップ(`cargo build` / `cargo test` / `cargo clippy`)は job レベルの `defaults.run.working-directory: app/src-tauri` に従っており個別指定していない。一貫性のため、新ステップも個別指定を省略し既存パターンに合わせる。

**4. `rustfmt.toml` は追加しない**
デフォルト設定で 53 箇所の差分はすべて解消可能であり、プロジェクト固有のスタイル要件は今のところ存在しない。将来スタイルの調整が必要になった時点で別 change として追加する。

**5. `cargo fmt --check` は `cargo build` / `cargo test` より前に配置する**
`/code-review` の指摘により、コンパイル不要で数秒で終わる `cargo fmt --check` を、数分かかる `cargo build` / `cargo test` より先に実行する構成に変更した。フォーマット違反だけの些細な問題で不要なビルド・テストが走ることを避け、フェイルファストにする。決定3で述べた「既存3ステップの構成に合わせる」方針とは、ステップの記述スタイル(`working-directory` を省略する等)を指しており、実行順序まで固定するものではない。

## Risks / Trade-offs

- [Risk] `cargo fmt` の一括適用により 12 ファイルに整形差分が入り、`git blame` の履歴が一時的に読みにくくなる → [Mitigation] 単一の "chore: cargo fmt" コミットに閉じ込め、コミットメッセージで明示する。将来 `git blame --ignore-revs-file` 等での除外を検討可能だが本changeのスコープ外とする。
- [Risk] フォーマット適用によって意図しない挙動変化が混入する可能性 → [Mitigation] フォーマット適用後に `cargo build` / `cargo test` / `cargo clippy --all-targets -- -D warnings` をローカルで実行し、フォーマット前と同じ結果になることを確認してからコミットする。
- [Risk] 今後 rustfmt のバージョンが上がりデフォルトスタイルが変わった場合、CI が突然赤くなる可能性 → [Mitigation] 現状は許容する(Non-Goal参照)。`dtolnay/rust-toolchain@stable` を使っているため既存の build/clippy ステップも同様のリスクを内包しており、本changeで新たに導入されるものではない。

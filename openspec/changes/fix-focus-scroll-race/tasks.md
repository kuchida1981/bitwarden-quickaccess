## 1. 実装

- [x] 1.1 `app/dist/app.js` にモジュールスコープの `suppressMouseEnterFocus` フラグを追加する
- [x] 1.2 `renderResults()` 内の `scrollIntoView` 呼び出し(app.js:542付近)の直前で `suppressMouseEnterFocus = true` をセットする
- [x] 1.3 `updateFocusRows()` 内の `scrollIntoView` 呼び出し(app.js:579付近)の直前で `suppressMouseEnterFocus = true` をセットする
- [x] 1.4 `resultsList` に `mousemove` リスナーを追加し、発火時に `suppressMouseEnterFocus = false` に戻す
- [x] 1.5 各行の `mouseenter` ハンドラ(app.js:522-534)の先頭で `suppressMouseEnterFocus` が真の場合は処理をスキップするようにする

## 2. 動作確認

- [x] 2.1 `cargo run` でアプリを起動し、検索結果を一覧のスクロール対象になる件数まで表示させる
- [x] 2.2 一覧内の途中の行にマウスカーソルを置いたまま↓キーを連打し、スクロールが発生する境目でもキー操作した行にフォーカスが留まり続けることを確認する(#128の再現手順の解消確認)
- [x] 2.3 マウスを実際に別の行へ動かした場合は、従来通りホバーでフォーカスが切り替わることを確認する(意図した挙動が壊れていないことの確認)
- [x] 2.4 `cargo test` / `cargo clippy --all-targets -- -D warnings` を実行し、既存のRust側テストに影響がないことを確認する

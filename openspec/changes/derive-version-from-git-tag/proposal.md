## Why

GitHub issue #78: `.github/workflows/release.yml` は公式リリースのビルド時にのみ、リリースタグの値で `Cargo.toml` の `version` を書き換えている(mainブランチへはコミットしない設計)。このため `Cargo.toml` 自体は最後に手動更新した値(現在 `1.0.0`)のまま更新されず、`cargo run`/`cargo tauri build` によるセルフビルドは常にこの古い値を `env!("CARGO_PKG_VERSION")` 経由でトレイメニューに表示し続ける。v1.1.0リリース作業時、この結果セルフビルド(`v1.0.0`表示)と実際のHomebrew版(`v1.1.0`)がSpotlight上で2つの同名アプリとして混在し、ユーザーが混乱するインシデントが発生した。

## What Changes

- `app/src-tauri/build.rs` で、ビルド時に `git describe --tags --always` を実行し、その結果を環境変数(`BWQA_DISPLAY_VERSION`)としてビルド先に渡す。
- `app/src-tauri/src/tray.rs` の `APP_VERSION` を `env!("CARGO_PKG_VERSION")` から `env!("BWQA_DISPLAY_VERSION")` に置き換える。
- タグちょうどのコミットでビルドした場合(公式リリースビルド)は `v1.1.0` のようにクリーンに表示され、タグより後のコミットでのセルフビルドは `v1.1.0-3-gabc1234` のような形式になり、正式リリースそのものではないことが一目で分かるようにする。
- `.git` が無い・`git`コマンドが無い環境向けに、`Cargo.toml` の値をそのまま使うフォールバックを用意する。
- `.github/workflows/release.yml` の `actions/checkout@v4` に、タグ情報を確実に取得できるよう `fetch-depth: 0` を設定する。「Sync Cargo.toml version with the release tag」ステップは**削除しない**(下記「補足」参照)。
- `CONTRIBUTING.md` のリリース手順に、トレイ表示とアプリバンドルのバージョンが別経路で決まることを補足する。

### 補足: 「Sync Cargo.toml version」ステップは削除しない

issue #78本文は当初この削除を「副次効果」として提案していたが、実装レビューの過程で誤りと判明した。`tauri.conf.json` に `version` フィールドが無いため、Tauriは `Cargo.toml` の `version` を**macOSアプリバンドル自体のメタデータ(Info.plistのCFBundleShortVersionString等、Finderの「情報を見る」に表示される値)**の情報源として使う。これは今回git describeベースに切り替えた「トレイメニュー内の表示文字列」とは完全に別の経路であり、このステップを削除すると、トレイ表示は直ってもバンドル自体のバージョンが将来のリリースすべてで`1.0.0`のまま固定されてしまう(まさに本changeが解決しようとしているインシデントの再発)。そのためこのステップは維持する。

## Capabilities

### New Capabilities
(なし)

### Modified Capabilities
- `about-and-branding`: 「トレイメニューでのアプリ名・バージョン表示」要件に、バージョン番号がgit tagから動的に導出されること(クリーンなタグビルド/開発中のセルフビルドで表示形式が異なること)を追記する。

## Impact

- `app/src-tauri/build.rs`: git describeによるバージョン導出ロジックの追加
- `app/src-tauri/src/tray.rs`: `APP_VERSION` の参照先変更、`about_item` のフォーマット調整(git describe形式は既に`v`を含むため二重にならないようにする)
- `.github/workflows/release.yml`: `fetch-depth: 0` の追加、`Swatinem/rust-cache` の削除(タスク4.4のレビューで発覚したキャッシュ起因の再発リスクへの対応。詳細はdesign.md参照)。「Sync Cargo.toml version」ステップは維持する(理由は上記補足を参照)。
- `CONTRIBUTING.md`: リリース手順の記述更新
- 破壊的変更なし。`Cargo.toml` の `version` フィールドは、アプリバンドルのメタデータ用としては引き続きリリースタグと同期される。トレイメニューの表示用途としては参照されなくなる。

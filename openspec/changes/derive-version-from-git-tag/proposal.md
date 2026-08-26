## Why

GitHub issue #78: `.github/workflows/release.yml` は公式リリースのビルド時にのみ、リリースタグの値で `Cargo.toml` の `version` を書き換えている(mainブランチへはコミットしない設計)。このため `Cargo.toml` 自体は最後に手動更新した値(現在 `1.0.0`)のまま更新されず、`cargo run`/`cargo tauri build` によるセルフビルドは常にこの古い値を `env!("CARGO_PKG_VERSION")` 経由でトレイメニューに表示し続ける。v1.1.0リリース作業時、この結果セルフビルド(`v1.0.0`表示)と実際のHomebrew版(`v1.1.0`)がSpotlight上で2つの同名アプリとして混在し、ユーザーが混乱するインシデントが発生した。

## What Changes

- `app/src-tauri/build.rs` で、ビルド時に `git describe --tags --always` を実行し、その結果を環境変数(`BWQA_DISPLAY_VERSION`)としてビルド先に渡す。
- `app/src-tauri/src/tray.rs` の `APP_VERSION` を `env!("CARGO_PKG_VERSION")` から `env!("BWQA_DISPLAY_VERSION")` に置き換える。
- タグちょうどのコミットでビルドした場合(公式リリースビルド)は `v1.1.0` のようにクリーンに表示され、タグより後のコミットでのセルフビルドは `v1.1.0-3-gabc1234` のような形式になり、正式リリースそのものではないことが一目で分かるようにする。
- `.git` が無い・`git`コマンドが無い環境向けに、`Cargo.toml` の値をそのまま使うフォールバックを用意する。
- `.github/workflows/release.yml` から不要になった「Sync Cargo.toml version with the release tag」ステップを削除する。`actions/checkout@v4` がタグ情報を確実に取得できるよう `fetch-depth: 0` を設定する。
- `CONTRIBUTING.md` のリリース手順から、上記削除したステップに関する記述を更新する。

## Capabilities

### New Capabilities
(なし)

### Modified Capabilities
- `about-and-branding`: 「トレイメニューでのアプリ名・バージョン表示」要件に、バージョン番号がgit tagから動的に導出されること(クリーンなタグビルド/開発中のセルフビルドで表示形式が異なること)を追記する。

## Impact

- `app/src-tauri/build.rs`: git describeによるバージョン導出ロジックの追加
- `app/src-tauri/src/tray.rs`: `APP_VERSION` の参照先変更、`about_item` のフォーマット調整(git describe形式は既に`v`を含むため二重にならないようにする)
- `.github/workflows/release.yml`: 「Sync Cargo.toml version」ステップの削除、`fetch-depth: 0` の追加
- `CONTRIBUTING.md`: リリース手順の記述更新
- 破壊的変更なし。`Cargo.toml` の `version` フィールド自体は今後も存在するが、表示用途としては参照されなくなる。

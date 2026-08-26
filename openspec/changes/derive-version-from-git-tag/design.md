## Context

`app/src-tauri/build.rs` は現在 `tauri_build::build()` を呼ぶだけ。`tray.rs` の `APP_VERSION` 定数は `env!("CARGO_PKG_VERSION")`(コンパイル時に `Cargo.toml` の `version` を埋め込む)を参照しており、`about_item` の表示に使われている。

`release.yml` は公式リリースビルド時のみ、チェックアウトしたワークスペース内で `Cargo.toml` を書き換えてからビルドしている(mainへのコミットはしない)。この結果、`Cargo.toml` 自体の値は事実上リリースの度に手動更新されない限り古いままになる。

## Goals / Non-Goals

**Goals:**
- ビルド時の実際のgit状態(タグ・コミット)から動的にバージョン文字列を導出し、公式リリースビルド・セルフビルドの両方で正確な表示にする。
- `.git` が無い環境やgitコマンドが無い環境でもビルド自体は失敗しないようにする(フォールバック)。
- `release.yml` の「Cargo.tomlその場書き換え」ステップを不要にし、リリースワークフローを単純化する。

**Non-Goals:**
- `Cargo.toml` の `version` フィールド自体の削除・完全な廃止(Cargoエコシステム上、`version`フィールドは必須であり残す。表示用途に使わなくなるだけ)。
- セルフビルド配布(Homebrew以外での配布方法)のサポート追加。

## Decisions

- **`build.rs` で `git describe --tags --always` を実行し、`cargo:rustc-env=BWQA_DISPLAY_VERSION=...` として埋め込む**。`--always` フラグにより、タグが1つも無いリポジトリでもコミットハッシュにフォールバックする(gitコマンド自体が使える前提での最終防御線)。
- **gitコマンドの実行に失敗した場合(`.git`が無い、gitが未インストール等)は `format!("v{}", env!("CARGO_PKG_VERSION"))` にフォールバックする**。`env!("CARGO_PKG_VERSION")` はbuild.rs自身のコンパイル時に解決されるため、追加の依存なしに使える。
- **`git describe` の出力は既に `v`(タグ名の接頭辞)を含むため、`tray.rs` 側で `format!("{} v{}", ...)` のように手動で `v` を追加している箇所を `format!("{} {}", ...)` に修正し、二重の `v` を防ぐ**。フォールバック時もこの前提に合わせて `v` を含む形で埋め込む。
- **`build.rs` に `cargo:rerun-if-changed` を追加し、`.git/HEAD` と `.git/refs/tags` の変更を監視する**。これが無いと、ソースコードを変更せずタグだけ打っても再ビルドされずバージョン表示が古いまま固定されてしまう。
- **`release.yml` の checkout ステップに `fetch-depth: 0` を設定する**。デフォルトの浅いcheckoutではタグ情報が不足し `git describe` が正しく動かない可能性があるため(issue本文の確認事項)。

## Risks / Trade-offs

- [`fetch-depth: 0` によりCIのcheckoutが多少遅くなる] → リリースワークフローは頻繁に実行されるものではなく、許容範囲と判断する。
- [開発者のローカル環境でタグを取得していない(`git fetch --tags`していない)clone状態だと、意図せず短いコミットハッシュ表示になる] → これは「正式リリースそのものではない」ことを示す意図した挙動であり問題ない。

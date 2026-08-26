## Context

`app/src-tauri/build.rs` は現在 `tauri_build::build()` を呼ぶだけ。`tray.rs` の `APP_VERSION` 定数は `env!("CARGO_PKG_VERSION")`(コンパイル時に `Cargo.toml` の `version` を埋め込む)を参照しており、`about_item` の表示に使われている。

`release.yml` は公式リリースビルド時のみ、チェックアウトしたワークスペース内で `Cargo.toml` を書き換えてからビルドしている(mainへのコミットはしない)。この結果、`Cargo.toml` 自体の値は事実上リリースの度に手動更新されない限り古いままになる。

## Goals / Non-Goals

**Goals:**
- ビルド時の実際のgit状態(タグ・コミット)から動的にバージョン文字列を導出し、公式リリースビルド・セルフビルドの両方で**トレイメニュー上の表示**を正確にする。
- `.git` が無い環境やgitコマンドが無い環境でもビルド自体は失敗しないようにする(フォールバック)。
- `release.yml` のタグ情報取得を確実にする(`fetch-depth: 0`)。

**Non-Goals:**
- `Cargo.toml` の `version` フィールド自体の削除・完全な廃止(Cargoエコシステム上必須であり残す)。
- `release.yml` の「Sync Cargo.toml version with the release tag」ステップの削除(下記Decisionsで詳述する理由により、これは本changeのスコープ外=維持する)。
- セルフビルド配布(Homebrew以外での配布方法)のサポート追加。

## Decisions

- **`build.rs` で `git describe --tags --always` を実行し、`cargo:rustc-env=BWQA_DISPLAY_VERSION=...` として埋め込む**。`--always` フラグにより、タグが1つも無いリポジトリでもコミットハッシュにフォールバックする(gitコマンド自体が使える前提での最終防御線)。
- **gitコマンドの実行に失敗した場合(`.git`が無い、gitが未インストール等)は `format!("v{}", env!("CARGO_PKG_VERSION"))` にフォールバックする**。`env!("CARGO_PKG_VERSION")` はbuild.rs自身のコンパイル時に解決されるため、追加の依存なしに使える。
- **`git describe` の出力は既に `v`(タグ名の接頭辞)を含むため、`tray.rs` 側で `format!("{} v{}", ...)` のように手動で `v` を追加している箇所を `format!("{} {}", ...)` に修正し、二重の `v` を防ぐ**。フォールバック時もこの前提に合わせて `v` を含む形で埋め込む。
- **`build.rs` に `cargo:rerun-if-changed` を追加し、`.git/HEAD` と `.git/refs/tags` の変更を監視する**。これが無いと、ソースコードを変更せずタグだけ打っても再ビルドされずバージョン表示が古いまま固定されてしまう。
- **`release.yml` の checkout ステップに `fetch-depth: 0` を設定する**。デフォルトの浅いcheckoutではタグ情報が不足し `git describe` が正しく動かない可能性があるため(issue本文の確認事項)。
- **「Sync Cargo.toml version with the release tag」ステップは削除せず維持する**(コードレビューで発見・当初のissue案から方針転換)。`tauri.conf.json` に `version` が無いため、Tauriは `Cargo.toml` の `version` を**アプリバンドル自体のメタデータ(Info.plistのCFBundleShortVersionString等)**の情報源として使う。これは今回git describeベースにした「トレイメニュー内の表示文字列」(`BWQA_DISPLAY_VERSION`)とは完全に別の経路であり、このステップを削除すると、トレイ表示は直ってもバンドル自体のバージョンが将来のリリースすべてで固定されてしまい、本changeが解決しようとしているインシデントが形を変えて再発する。issue #78本文はこの二重の情報源を認識しておらず、「削除可能」という提案は誤りだった。
- **`release.yml` から `Swatinem/rust-cache` を削除する**(タスク4.4のレビュー中に発見)。`rerun-if-changed`はファイルのmtime比較に依存するため、`Cargo.lock`が変わらない連続リリース(依存関係の変更が無いパッチリリース等)でキャッシュされたビルド成果物がヒットすると、`.git/HEAD`のmtimeが「キャッシュ復元時点」基準で古いと判定され、`build.rs`が再実行されずに前回リリース時点の古いバージョンが埋め込まれてしまうリスクがある。これはまさに本changeが解決しようとしている問題の再発になりかねないため、リリースビルドに限りキャッシュを使わないことで確実性を優先する(リリースは頻繁に行うものではないため、ビルド時間の増加は許容する)。

## Risks / Trade-offs

- [`fetch-depth: 0` によりCIのcheckoutが多少遅くなる] → リリースワークフローは頻繁に実行されるものではなく、許容範囲と判断する。
- [開発者のローカル環境でタグを取得していない(`git fetch --tags`していない)clone状態だと、意図せず短いコミットハッシュ表示になる] → これは「正式リリースそのものではない」ことを示す意図した挙動であり問題ない。
- [`cargo:rerun-if-changed` によるディレクトリ(`.git/refs/tags`)監視は、実機検証の結果、新規タグ追加だけでは確実に再ビルドをトリガーしないことが判明した(Cargoのディレクトリ監視自体の制約による)。ソースコードの変更を伴わずタグだけ打ってから明示的な操作(`build.rs`のtouch等)無しに再ビルドした場合、古いバージョン表示のまま固定される可能性がある] → 実用上の主な用途(CIでのリリースビルド、通常のコミットを伴う開発ビルド)では `.git/HEAD` の変更で確実に再トリガーされるため影響は限定的。「タグだけ打って即座に再ビルド」という狭いローカル開発シナリオのみの既知の制限として許容する。

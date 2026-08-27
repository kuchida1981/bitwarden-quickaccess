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
- **`git describe` の出力は(タグが参照可能な場合)既に `v` を含むため、`tray.rs` 側で `format!("{} v{}", ...)` のように手動で `v` を追加している箇所を `format!("{} {}", ...)` に修正し、二重の `v` を防ぐ**。ただし `--always` は参照可能なタグが1つも無い場合に `v`無しの生のコミットハッシュへフォールバックするため(コードレビューで発覚)、`build.rs` 側で「`v`で始まっていなければ付与する」正規化を行い、`BWQA_DISPLAY_VERSION` が常に `v` から始まる不変条件を保証する。
- **`build.rs` に `cargo:rerun-if-changed=<repo_root>/.git/HEAD` のみを追加する**(`.git/refs/tags` の監視は当初案から撤回)。新しいコミットのたびに `.git/HEAD` は更新されるため、通常の開発フロー・CIでの新規checkoutでは確実に再ビルドがトリガーされる。`.git/refs/tags` も併せて監視する案があったが、コードレビューで2点の問題が判明したため撤回した: (1) 実機検証の結果、新規タグ追加だけでは確実に再ビルドをトリガーしない(Cargoのディレクトリ監視自体の制約)、(2) タグの無いshallow clone等 `.git/refs/tags` が存在しない環境では、Cargoの仕様上「存在しないパスは常に変更ありとみなす」ため**逆に毎回無条件で再ビルドが走ってしまう**(恩恵が薄い上に新たな害がある)。「タグだけ打って即座に再ビルドしてもバージョン表示が更新されない」という狭いローカル開発シナリオは、既知の制限として許容する(下記Risks参照)。
- **`release.yml` の checkout ステップに `fetch-depth: 0` を設定する**。デフォルトの浅いcheckoutではタグ情報が不足し `git describe` が正しく動かない可能性があるため(issue本文の確認事項)。
- **「Sync Cargo.toml version with the release tag」ステップは削除せず維持する**(コードレビューで発見・当初のissue案から方針転換)。`tauri.conf.json` に `version` が無いため、Tauriは `Cargo.toml` の `version` を**アプリバンドル自体のメタデータ(Info.plistのCFBundleShortVersionString等)**の情報源として使う。これは今回git describeベースにした「トレイメニュー内の表示文字列」(`BWQA_DISPLAY_VERSION`)とは完全に別の経路であり、このステップを削除すると、トレイ表示は直ってもバンドル自体のバージョンが将来のリリースすべてで固定されてしまい、本changeが解決しようとしているインシデントが形を変えて再発する。issue #78本文はこの二重の情報源を認識しておらず、「削除可能」という提案は誤りだった。
- **`release.yml` の `Swatinem/rust-cache` は削除せず維持する**(タスク4.4のレビュー中に一度削除を検討したが、根拠不十分と判断し撤回)。当初「`Cargo.lock`が変わらない連続リリースでキャッシュされた古いビルド出力が再利用され、バージョンが古いまま固定されるのではないか」という懸念を立てたが、これは検証(実際のCI環境での実行結果の確認)を伴わない推測に留まっていた。`actions/checkout` は実行のたびに必ずフレッシュなクローンを行うため `.git/HEAD` は毎回新しいmtimeを持つ一方、`actions/cache`(`Swatinem/rust-cache`が内部で使う)はtar展開時に一般的に元のmtime(キャッシュ保存時点の古い値)を保持する。したがって、フレッシュな `.git/HEAD` は復元されたキャッシュ成果物より常に新しいと判定されるはずで、`rerun-if-changed` は正しく再実行をトリガーし続けると考えられる。根拠の薄いリスク回避のために毎リリースのビルド時間を恒久的に増やすのは見合わないと判断し、キャッシュは維持する。

## Risks / Trade-offs

- [`fetch-depth: 0` によりCIのcheckoutが多少遅くなる] → リリースワークフローは頻繁に実行されるものではなく、許容範囲と判断する。
- [開発者のローカル環境でタグを取得していない(`git fetch --tags`していない)clone状態だと、意図せず短いコミットハッシュ表示になる] → これは「正式リリースそのものではない」ことを示す意図した挙動であり問題ない。
- [`Swatinem/rust-cache` 維持の判断根拠(tarのmtime保持挙動)は、実際のGitHub Actions実行環境で経験的に検証したものではなく、一般的なtar/actions/cacheの挙動からの推論に基づく] → もし将来、実リリースでトレイのバージョン表示が古いまま(前回リリースの値)になる不具合が観測された場合は、真っ先にこの`rust-cache`とキャッシュされた`target/`のmtimeの実際の挙動を疑い、再調査すること。
- [`.git/HEAD` のみを監視対象とするため、ソースコードの変更・新規コミットを伴わずタグだけ打って明示的な操作(`build.rs`のtouch等)無しに再ビルドした場合、古いバージョン表示のまま固定される] → 実用上の主な用途(CIでのリリースビルド、通常のコミットを伴う開発ビルド)では `.git/HEAD` の変更で確実に再トリガーされるため影響は限定的。「タグだけ打って即座に再ビルド」という狭いローカル開発シナリオのみの既知の制限として許容する(`.git/refs/tags` も監視する案を検討したが、上記Decisionsの通り撤回した)。

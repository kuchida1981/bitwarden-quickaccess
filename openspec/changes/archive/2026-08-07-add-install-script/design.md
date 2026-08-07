## Context

`bin/bw-quickaccess` は自身の絶対パスから `BWQA_ROOT`(`bin/` の親ディレクトリ)を算出し、`lib/*.sh` を `source "$BWQA_ROOT/lib/xxx.sh"` の形で相対 source している。`lib/*.sh` はいずれも shebang を持たず「source される前提」で書かれており(`lib/common.sh` 冒頭コメントにも明記)、path 依存も `BWQA_ROOT` 経由の source 以外には存在しない。

このため、`bin/bw-quickaccess` を単体でコピーしても `lib/` を発見できず動作しない。一方、`lib/fields.sh` / `lib/search.sh` は fzf の `--bind execute-silent(...)` から `$BWQA_SELF`(自分自身の絶対パス)を `__copy-field` サブコマンド付きで再帰的に呼び出している。`BWQA_SELF` は `BWQA_ROOT` とは独立に「起動時の自分の絶対パス」を指すだけなので、実行ファイルの配置場所に依存しない。

このプロジェクトにはこれまでリリースタグ・GitHub Release が一度も存在しない(初回リリースをこの change で作ることになる)。

## Goals / Non-Goals

**Goals:**
- `curl -fsSL <install.sh の URL> | bash` で bw-quickaccess をユーザー権限インストールできるようにする
- インストール先ルートを `--prefix` オプションで変更できるようにする
- `bin/bw-quickaccess` / `lib/*.sh` の既存ランタイムロジック(vault検索・フィールドコピー・session管理等)は変更しない(バンドルは既存の source 連鎖と結果的に等価な連結処理にとどめる)。`--version`/`-v` オプションの追加のみ例外として許容する
- GitHub Release 公開に連動してビルド済みバンドルを自動的にアセット添付する
- `install.sh` を再実行するだけで既存インストールを最新版に更新できるようにする(専用の update スクリプトは設けない)

**Non-Goals:**
- Homebrew tap 化(issue #7 の候補案だが、今回はスコープ外。将来的な拡張として残す)
- Windows(ネイティブ)対応の install.sh(現状 bw-quickaccess 自体が macOS / Linux デスクトップ環境のみ対応のため)
- シェル rc ファイル(`.bashrc`/`.zshrc` 等)の自動編集(PATH 追加はメッセージ表示のみに留める)
- アンインストールスクリプト(README にコマンド例を書くのみで、スクリプト自体は今回のスコープ外)
- `video-ratings/scripts/update.sh` のような専用アップデータ(DBマイグレーション・systemd再起動等を伴う)。bw-quickaccess はステートレスな単一ファイル配布のため、install.sh の再実行で上書きするだけで十分

## Decisions

### 1. 配布形式: ビルド時バンドル(単一ファイル化)

`script/build.sh` を新設し、`lib/common.sh` → `preflight.sh` → `clipboard.sh` → `session.sh` → `search.sh` → `fields.sh` の順に内容を連結し、`bin/bw-quickaccess` 本体から `source` 行を除いた部分と合わせて単一の実行可能スクリプト(出力ファイル名 `bw-quickaccess`)を生成する。

現在の起動時 `source` 連鎖は「同一シェルコンテキストで逐次読み込んで実行する」処理であり、ビルド時の単純な `cat` 連結と意味的に等価(sourceされる側に相対パス依存や多重source防止ガードが無いことを確認済み)。`BWQA_SELF` の解決ロジックは変更不要(バンドル後もその実行ファイル自身の絶対パスを指すため引き続き機能する)。

これにより、インストール後は `$PREFIX/bin/bw-quickaccess` に**単一ファイルを置くだけ**でよくなり、以下が全て不要になる:
- symlink 先解決(`bin/bw-quickaccess` が symlink 経由で起動された場合の `dirname` 誤解決問題)
- `bin/` を直接叩くだけの wrapper スクリプト
- `lib/` 一式の配置場所(`share/` は本来アーキテクチャ非依存データ向けであり、実行に使うライブラリの置き場として不自然という指摘があった。バンドルによりこの配置問題自体が消滅する)

**代替案として検討したもの:**
- *wrapper スクリプト + symlink*: `bin/bw-quickaccess` 本体は無改修で済むが、symlink 経由起動時の `BASH_SOURCE[0]` が symlink 自体のパスになり `dirname` 解決が壊れる。本体側に symlink 解決ロジック(BSD `readlink` は `-f` 非対応なためループ実装が必要)を追加する必要があり、複雑さが増す
- *git clone による配布*: install.sh がリポジトリ全体を clone する方式。実装は単純だが `test/`・`openspec/` 等の開発用ディレクトリまで持ち込まれ、git 依存が install 時に発生する
- *個別ファイルを raw.githubusercontent.com から curl*: `lib/*.sh` の増減に応じて install.sh 側のファイルリストを都度更新する必要があり保守性が低いため却下

### 2. リリース運用: 人間主導(パターンB)

タグ作成・GitHub Release 作成は人間が `gh release create <tag> --generate-notes` で行う(新規タグ指定・既存タグ指定のどちらも可)。CI のトリガーは `on: release: types: [published]` とし、`push: tags` は使わない。

理由: `gh release create` に既存タグを指定した場合、タグの push イベントは発火しない(タグは既に push 済みのため)。`push: tags` をトリガーにすると、この「既存タグへの release 作成」ケースを CI が拾えなくなる。`release: published` イベントであれば、新規タグ経由・既存タグ経由のどちらでも release が公開された時点で確実に発火する。また draft のまま保存している間は `published` イベントが発火しないため、「下書き→レビュー→公開」の運用とも自然に噛み合う。

CI(`.github/workflows/release.yml`)がやることは以下のみ:
1. `github.event.release.tag_name` が指す commit を checkout
2. `script/build.sh` でバンドルをビルド
3. `gh release upload <tag_name> bw-quickaccess` で、**既に人間が作成済みの** release にアセットを追加する(release 自体は作らない)

**代替案として検討したもの:**
- *タグ push トリガー + CI が release 自体も作成(パターンA)*: 参考にした `video-ratings` プロジェクトではこの方式(`push: tags` トリガー + `softprops/action-gh-release` で release 作成とアセット添付を一括実行)を採用している。人間の操作が `git push --tags` のみで完結する利点があるが、今回は「タグ・release の作成はどちらも人間が行い、リリースノート等をコントロールしたい」という明示的な希望があったため、意図的にパターンBを採用する。プロジェクト間で運用が異なる点は許容する

### 3. install.sh の挙動

- 実行方法: `curl -fsSL <raw github URL>/install.sh | bash`(`-s -- --prefix ...` でオプション付与も可能な形にする)
- デフォルトインストール先: `~/.local`(`--prefix` オプションで変更可能)。バンドルが単一ファイルのため実質使うのは `$PREFIX/bin` のみ
- バージョン解決:
  - デフォルト(バージョン指定なし)は GitHub API 呼び出し無しで `https://github.com/<owner>/<repo>/releases/latest/download/bw-quickaccess` を直接 curl する(GitHub が提供する "latest" リダイレクトショートカットを利用)
  - バージョン指定オプションを付けた場合のみ `https://github.com/<owner>/<repo>/releases/download/<version>/bw-quickaccess` に切り替える
- 取得したファイルを `$PREFIX/bin/bw-quickaccess` に配置し `chmod +x`
- インストール完了後、`$PREFIX/bin` が `PATH` に含まれていなければ警告メッセージを表示する(コピペ用の `export PATH=...` コマンド例を提示するのみで、シェル rc ファイルの自動編集は行わない)
- install.sh 自体の実行に git は不要(リリースアセットを直接 curl するだけのため)

**参考にした既存実装:** `video-ratings` プロジェクトの `scripts/update.sh` にある `curl https://api.github.com/repos/<repo>/releases/latest | grep tag_name` によるバージョン解決パターン。ただし bw-quickaccess ではデフォルトケースに限り GitHub の `releases/latest/download/<asset>` ショートカット URL を使うことで API 呼び出し自体を省略する。

### 4. アップデート方法とバージョン確認

**アップデート = install.sh の再実行。** video-ratings は systemd サービス・PostgreSQL マイグレーション・複数バージョンの releases ディレクトリ管理を伴うため専用の `update.sh`(`scripts/update.sh` → `/usr/local/bin/video-ratings-update`)を持つが、bw-quickaccess はステートレスな単一ファイル配布であり、マイグレーションもサービス再起動も不要。そのため専用スクリプトは作らず、**同じ install.sh を同じ(または異なる)`--prefix`/`--version` で再実行すれば、既存ファイルを新しいバンドルで上書きするだけで更新が完了する**設計にする。

ユーザーが「更新されたか」「何のバージョンから何に上がったか」を確認できるよう、以下を追加する:

- `bin/bw-quickaccess` に `--version`/`-v` オプションを追加し、`bw-quickaccess <version>` の形式でバージョン文字列を出力する
- バージョン文字列はビルド時に埋め込む。`script/build.sh` は環境変数 `VERSION` を受け取り、指定があればバンドル先頭に `BWQA_VERSION="$VERSION"` を書き込む。指定が無いローカルビルドでは `bin/bw-quickaccess` 側のデフォルト(`: "${BWQA_VERSION:=dev}"` のような未設定時デフォルト)により `dev` と表示される
- `.github/workflows/release.yml` は `script/build.sh` 実行時に `VERSION="${{ github.event.release.tag_name }}"` を渡し、リリースタグをそのままバージョン文字列として埋め込む
- `install.sh` は上書き前に、既にインストール済みの実行ファイルがあれば `"$PREFIX/bin/bw-quickaccess" --version` で現在のバージョンを取得し、ダウンロード後に新しいバージョンと合わせて `vX.Y.Z → vA.B.C に更新しました` のように表示する。新規インストール時は単に `vA.B.C をインストールしました` と表示する
- 「最新版かどうかの事前チェック」(現在バージョンと最新版を比較してスキップする最適化)は行わない。デフォルトケースで GitHub API 呼び出しを避ける設計(3節)と矛盾するため、常にダウンロード→上書きするシンプルな動作に統一する

**代替案として検討したもの:**
- *video-ratings 同様の専用 `update.sh` を用意*: バージョンごとの releases ディレクトリ管理・ロールバックが可能になるが、単一ファイル配布には過剰。却下
- *`git describe --tags` でビルド時にバージョンを自動解決*: CI が shallow checkout かつ `github.event.release.tag_name` を確実に持っているため、`git describe` に頼らず明示的に `VERSION` を渡す方が単純で確実

## Risks / Trade-offs

- **[Risk]** `script/build.sh` の連結順序を誤ると実行時エラーになるが、テストで検知しにくい → **Mitigation**: ビルド成果物に対しても `bash -n` と `shellcheck` を CI(既存の `ci.yml` または `release.yml`)で実行し、構文・静的解析エラーを検知する
- **[Risk]** `lib/*.sh` に将来 shebang や相対パス依存が追加されると、単純連結が壊れる → **Mitigation**: `lib/common.sh` の「source される前提」コメントを踏襲し、新規 lib ファイル追加時のガイドラインとして CLAUDE.md 等に明記することを検討する(この change のスコープでは対応しないが Open Questions に記載)
- **[Risk]** 初回リリース(`v0.1.0` 想定)を切る作業自体がこの change に依存しており、リリース運用が実際に機能するかは実リリースを切るまで検証できない → **Mitigation**: tasks.md に「テストタグでの動作確認」を含め、本番タグを切る前に CI の動作を検証する
- **[Trade-off]** パターンB(人間主導のリリース)は `video-ratings` のパターンA(CI主導)と運用が異なり、プロジェクトをまたいだ一貫性がない → 許容する(明示的な選択のため)
- **[Risk]** ローカルビルド(`VERSION` 未指定)はすべて `dev` と表示され、複数のローカルビルド同士を区別できない → **Mitigation**: ローカル開発時の区別は本 change のスコープ外とする(必要になった場合は commit hash 等の付加を別途検討)
- **[Risk]** アップデート時に「最新版かどうかの事前チェック」を行わないため、既に最新版でも常に再ダウンロード・上書きが発生する → **Mitigation**: バンドルはファイルサイズが小さく、頻繁に実行する操作でもないため許容する

## Open Questions

- `script/build.sh` の配置ディレクトリ名は `script/` と `scripts/`(video-ratings は `scripts/`)のどちらにするか。本 design では単数形 `script/` としたが、命名は tasks 実装時に確定する
- lib ファイル追加時に `script/build.sh` の連結順序リストを更新し忘れるリスクへの恒久対策(例: `lib/*.sh` を glob で自動的に拾う設計にできないか)は実装時に検討する

## Context

`lib/*.sh` は関数ベースのシェルスクリプト群で、外部コマンド(`bw`/`fzf`/`jq`/`security`/`secret-tool`/clipboard コマンド)への依存度が箇所によって異なる。

- `lib/common.sh` の `bwqa_version_ge()` は外部コマンドに依存しない純粋な文字列/数値比較
- `lib/session.sh` の `bwqa_session_ttl_expired()` はファイル読み取りと `date +%s` のみに依存する軽量ロジック
- `lib/search.sh` / `lib/fields.sh` の一部関数(`bwqa_fetch_items`, `bwqa_build_field_rows`, `bwqa_get_item_summary`)は内部ラッパー `bwqa_bw()`(`lib/session.sh` 定義)経由で `bw` を呼ぶ。`bwqa_bw()` 自体はセッション失効時の再認証リトライを持つ
- `lib/fields.sh` の `bwqa_copy_field_internal()`(`__copy-field` サブコマンドの実体)は `bw get <field>` の直接呼び出しと `bwqa_copy_to_clipboard`(`lib/clipboard.sh`、`BWQA_CLIPBOARD_CMD_ARR` 経由)に依存する
- `lib/preflight.sh` は `command -v`・`uname`・環境変数(`WAYLAND_DISPLAY`/`DISPLAY`)によるコマンド有無・OS・表示サーバー判定が中心

ローカル環境・CI 環境ともに `bats-core` と `shellcheck` は未導入で、`.github/` ディレクトリも存在しない。この change でテスト基盤と CI を新規に立ち上げる。

本 design は `/opsx:explore` での合意事項(モック戦略・bats 導入方法・CI matrix 構成)を実装可能な形に落とし込む。

## Goals / Non-Goals

**Goals:**
- `lib/*.sh` の純粋ロジック・分岐ロジックに対する bats-core 単体テストを追加する
- GitHub Actions で構文チェック(`bash -n`)・静的解析(`shellcheck`)・単体テスト(`bats`)を `macos-latest` / `ubuntu-latest` の両方で実行する
- 外部コマンドへの依存を、関数スタブと PATH ダミー実行ファイルの適材適所の使い分けでテスト可能にする

**Non-Goals:**
- `bw unlock` や実際の vault アクセスを伴う結合テスト(`bwqa_unlock`, `bwqa_get_session` の session 取得成功パス)は対象外。テスト用 Bitwarden アカウントの用意は別 issue とする
- fzf の実際の対話的画面(`bwqa_run_search_screen`, `bwqa_run_field_screen` の fzf 呼び出し自体)のテストは対象外。これらの関数内で使われている jq 整形部分(`bwqa_build_field_rows` 等)のみを対象とする
- Linux ヘッドレス/SSH 専用環境のサポート追加は対象外(README の既存スコープ外記述のまま)
- カバレッジ計測ツールの導入は対象外

## Decisions

### 1. モック戦略: 関数スタブ中心 + 検出ロジックのみ PATH ダミー

**決定**: `bwqa_bw()` など内部ラッパー関数はテスト内で `bwqa_bw() { ... }` として再定義(シャドーイング)し、jq 整形ロジックを狙い撃ちでテストする。一方、`bwqa_check_core_tools`(`bw`/`jq`/`fzf` の有無)、`bwqa_check_fzf_version`、clipboard コマンド検出(`bwqa_detect_clipboard_cmd`)は、検出ロジックそのものが検証対象のため、一時ディレクトリを `PATH` の先頭に追加してダミー実行可能ファイルを置く方式でテストする。

**理由**: `bwqa_fetch_items()` 等の関心事は「`bw` の生出力を jq でどう整形するか」であり、`bwqa_bw()` 自身のリトライ・エラーハンドリングまで模倣した `bw` フェイクを書くのは過剰。関数スタブなら fixtures の JSON をそのまま返すだけで済み、テストが壊れにくい。一方 preflight のコマンド検出は `command -v` の結果そのものが振る舞いの本体なので、関数スタブでは検証にならず PATH モックが必須。

**代替案として却下**: 全面 PATH モック(issue 本文の示唆に近い)。`bw` の実際の出力形式を模倣するフェイクスクリプトが必要になり、Bitwarden CLI のバージョンアップで出力形式が変わるたびにテストが壊れるリスクが高い。

**実装時の補足(「コマンドが存在しない」ケースの決定性)**: `bwqa_test_stub_setup` は既存 PATH の先頭に stub dir を追加するだけなので、実行機に本物の `bw`/`jq`/`fzf`/`pbcopy` 等が実際にインストールされていると「存在しない」ケースを正しく再現できない(実際、開発機・CI ランナーいずれも `jq` 等が別 PATH に既に入っている)。このため `test/helpers/stub.bash` に `bwqa_test_stub_path_only()` を追加し、PATH を stub dir のみに制限しつつ `awk`(`bwqa_check_fzf_version` が内部で使用)は実体へのパススルースタブとして用意することで、ホスト環境に依存しない決定的な「存在しない」テストを実現した。呼び出し順序の制約(`bwqa_test_stub_cmd` 自体が `chmod` を要するため、必要なダミーコマンドを先にすべて作ってから `bwqa_test_stub_path_only` を呼ぶ)はヘルパーのコメントに明記した。

**実装時に発見・修正したバグ(コードレビュー指摘)**: `bwqa_test_stub_cmd` が生成するダミー実行ファイルは当初 `#!/usr/bin/env bash` シェバンを使っていたが、`bwqa_test_stub_path_only` で PATH を stub dir のみに制限すると `env` が `bash` を解決できず `env: bash: No such file or directory`(exit 127)で失敗することが判明した(`bwqa_test_stub_path_only` を実際に使うテストは当時すべて `command -v` ベースの判定のみで、スタブスクリプトの実行そのものを経由していなかったため潜在化していた)。修正として、`bwqa_test_stub_cmd` はテスト開始時点で解決した bash の絶対パス(`BWQA_TEST_BASH_PATH`)をシェバンに直接埋め込むよう変更し、PATH 制限下でも `env` を経由せず確実に起動できるようにした。あわせて `bwqa_test_stub_path_only` のパススルー対象コマンドを `awk` 固定から可変長引数で追加できる形に一般化し(`bwqa_test_stub_path_only sed cut` のように呼べる)、`preflight.bats` の「bw/jq/fzf が揃っていれば成功する」テストを `bwqa_test_stub_path_only` 経由の実行に変更してこの経路(スタブスクリプトの実行)を実際に踏む回帰テストにした。

### 2. bats-core / shellcheck の導入: パッケージマネージャ経由(vendoring しない)

**決定**: ローカルは `brew install bats-core shellcheck`(macOS 前提。README にコマンドを明記)。CI は `macos-latest` ランナーで `brew install bats-core shellcheck`、`ubuntu-latest` ランナーは `shellcheck` が標準搭載済みのためインストール不要、`bats` は `apt-get install -y bats` または公式 GitHub Action(`bats-core/bats-action` 等)を使う。

**理由**: リポジトリは public のままの運用が前提であり、バージョン固定の厳密さより導入の単純さを優先した。git submodule での vendoring は管理コスト(サブモジュール更新・オフライン実行対応)に見合わないと判断。

**代替案として却下**: git submodule vendoring。バージョンを完全固定できる利点はあるが、初期セットアップの複雑さがこの規模のプロジェクトには過剰。

### 3. CI matrix: `macos-latest` + `ubuntu-latest`

**決定**: 2 ランナーの matrix で `bash -n` → `shellcheck` → `bats test/lib/*.bats` を実行する。

**理由**: リポジトリが public であるため、GitHub Actions は Free プランでも全 OS ランナー(macOS 含む)が無料枠の対象外(時間無制限)で利用できる。private 化の予定はないため、macOS ランナーのコスト面の懸念はない。`lib/preflight.sh` の OS 分岐ロジック(macOS/Linux/Wayland/X11)を両方の実 OS 上で検証できる点も matrix 採用の後押しになる。

**留意**: 将来 private 化する場合は macOS ランナーの消費レートが 10 倍になるため、その時点で matrix 構成の見直しが必要になる(このリスクは Risks / Trade-offs に記載)。

### 4. テストディレクトリ構成

```
test/
├── lib/
│   ├── common.bats       # bwqa_version_ge()
│   ├── session.bats      # bwqa_session_ttl_expired()
│   ├── search.bats       # bwqa_fetch_items()(bwqa_bw スタブ)
│   ├── fields.bats       # bwqa_build_field_rows(), bwqa_get_item_summary(),
│   │                      # bwqa_copy_field_internal()
│   └── preflight.bats    # bwqa_check_core_tools, bwqa_check_fzf_version,
│                          # bwqa_detect_platform, bwqa_detect_clipboard_cmd
├── fixtures/
│   ├── bw-list-items.json   # bw list items 相当のサンプル(type==1/type!=1 混在)
│   └── bw-get-item.json     # bw get item 相当のサンプル(password/username/totp 有無違い)
└── helpers/
    └── stub.bash          # PATH ダミーコマンド生成・関数スタブの共通ヘルパー(setup() から source)
```

各 `.bats` ファイルは対象の `lib/*.sh` を直接 `source` する(`bin/bw-quickaccess` は経由しない)。`test/helpers/stub.bash` に、一時 PATH ディレクトリの作成・ダミー実行ファイル生成・teardown での PATH 復元を共通化する。

### 5. `bwqa_copy_field_internal()` のテスト方針

**決定**: `BWQA_CLIPBOARD_CMD_ARR` を差し替え可能な配列としてそのまま利用し(teardown 不要、単に配列にダミーコマンドを設定するだけ)、`bw get <field>` 呼び出し部分は関数スタブではなく PATH ダミー `bw` を使う。

**理由**: この関数は `bw get username|password|totp` を直接呼んでおり `bwqa_bw()` を経由しない。関数境界がないため PATH モックが最も自然。エラーログ(`BWQA_ERROR_LOG_FILE`)への書き込み内容を検証することで、値取得失敗時・不正な field 名指定時の挙動を確認する。

### 6. shellcheck の除外ルール: `.shellcheckrc`(`SC2034` は除く)+ ピンポイントのインライン disable

**決定**: リポジトリ直下に `.shellcheckrc`(`shell=bash`、`disable=SC1091,SC2016,SC2329`)を追加する。`lib/*.sh` と `test/helpers/stub.bash`・`test/lib/*.bats` は他ファイルから source される前提の設計であり、単体で shellcheck にかけると shebang 不在(SC2148。`shell=bash` で解消)・動的パスによる追跡不可(SC1091)・意図的なシングルクォート(`bwqa_test_stub_cmd` の遅延展開設計、SC2016)・関数スタブが他ファイル側から間接的に呼ばれることによる「未使用関数」誤検知(SC2329)が構造的に発生することを実装時に確認した。これらはプロジェクトの sourcing アーキテクチャ・テスト用モック設計に起因する既知の誤検知としてグローバルに無効化する。

`SC2034`(未使用変数)はグローバル無効化の対象から外した(コードレビューで「design.md 自身が『将来の本当の unused variable バグ等を検知する能力をなるべく損なわないように』と明言しているのに、その検知能力そのものを丸ごと殺す SC2034 のグローバル無効化を追加していて矛盾している」と指摘され、妥当と判断して修正)。代わりに:
- プロダクションコード(`bin/bw-quickaccess` + `lib/*.sh`)は `shellcheck -x bin/bw-quickaccess` で単一エントリポイント経由のクロスファイル解析を行う。`-x` はソースされた全ファイルを実際に辿って解析するため、cross-file で消費される定数(`BWQA_SESSION_TTL_SECONDS` 等)を正しく「使用済み」と認識でき、`SC2034` を無効化しなくても誤検知なく通る(実装時に確認済み)。これにより、将来 lib/*.sh に本当の unused variable(タイポ等)が紛れ込んだ場合は CI で検知できる。
- `test/helpers/stub.bash` は `export` している変数群(`BWQA_LIB_DIR`/`BWQA_CACHE_DIR` 等)については、shellcheck が export を「ファイル外での使用」とみなして自動的に `SC2034` を出さないため、追加対応不要だった。
- `test/lib/fields.bats` の関数スタブ内で設定する `BWQA_OS_KIND`/`BWQA_CLIPBOARD_CMD_ARR`(`bwqa_copy_field_internal` 側からのみ間接参照され、export もされていない)は、該当行に `# shellcheck disable=SC2034` をピンポイントで付与した。

一方、`lib/fields.sh` の `bwqa_run_field_screen`/`bwqa_copy_field_internal` にあった `export BW_SESSION`/`BWQA_ITEM_ID` を subshell 内で行うパターン(fzf の `execute-silent` 経由で再起動される子プロセスへ環境変数を継承させる意図的な設計)は SC2030/SC2031/SC2153 を発火させたが、これは特定関数に固有の設計判断であり誤検知の性質もプロジェクト全体には一般化できないため、該当箇所にのみ `# shellcheck disable=...` を付与した(`.shellcheckrc` には追加しない)。

**理由**: `.shellcheckrc` でのグローバル無効化は、sourcing アーキテクチャに起因して全ファイルで一様に発生する誤検知(SC1091/SC2016/SC2329)に限定し、`SC2034` のような「本当のバグを検知できる」チェックは可能な限り有効なまま保つ(`-x` によるクロスファイル解析、export の活用、少数箇所のインライン disable で対応する)ことで、将来の unused variable バグ等を検知する能力をなるべく損なわないようにした。

**CI/ローカルでの呼び出し方**: プロダクションコードとテストコードで呼び出しを分ける。
```
shellcheck -x bin/bw-quickaccess
shellcheck test/helpers/*.bash test/lib/*.bats
```

## Risks / Trade-offs

- [Risk] `bw` の実出力フォーマット変更を fixtures が追従できず、テストは通るが実環境で壊れる sync 漏れが起きうる → [Mitigation] fixtures 作成時に実際の `bw list items --pretty` 等の出力を参考にする旨をテストコード内コメントで明記し、大きな bw バージョンアップ時は fixtures の見直しをタスク化する
- [Risk] PATH ダミーコマンド方式は bats の `setup`/`teardown` の実装漏れがあると他テストに影響(PATH 汚染)しうる → [Mitigation] `test/helpers/stub.bash` に PATH 復元を一元化し、各 `.bats` は個別に PATH 操作を書かない
- [Risk] macOS ランナーは起動・キューが Linux より遅く、CI のフィードバック速度が落ちる → [Mitigation] 許容(課金面の懸念がないため速度は許容範囲とする)
- [Risk] 将来 private 化した場合、macOS ランナーの消費レートが 10 倍になり無料枠を圧迫する → [Mitigation] private 化を検討する際に CI matrix 構成(macOS 除外や実行頻度の見直し)を再検討する
- [Risk] `push` と `pull_request` を両方 `on:` に設定すると、同一リポジトリ内で feature ブランチに push するたびに(open な PR があれば)同じ commit に対して matrix が二重実行される → [Mitigation] `push` トリガーを `branches: [main]` に限定し、feature ブランチへの push は `pull_request` イベントのみで検証する(コードレビューで指摘され修正)
- [Risk] TTL 境界値テストで `date +%s` をテスト側と `bwqa_session_ttl_expired` 内部側の2箇所で個別に評価しているため、境界に近い秒数(TTL-1秒)だと実行タイミングの秒またぎでフレーキーになりうる → [Mitigation] 「TTL未満」ケースの余裕を TTL-5秒に広げてレース耐性を持たせた(コードレビューで指摘され修正。「TTLちょうど」「TTL超過」側は秒またぎが起きても方向的に安全なため対応不要)
- [Risk] shellcheck のバージョンによって関数スタブ(cross-file 経由でのみ呼ばれる)への誤検知の報告名が異なる(ローカルの 0.11.0 は SC2329、GitHub Actions ubuntu-latest 標準搭載版は SC2317)ため、片方だけ無効化すると CI 環境で新たに失敗しうる → [Mitigation] 実際に PR の CI で SC2317 の発生を確認し、`.shellcheckrc` に SC2329 と併せて追加した
- [Risk] `test/lib/*.bats` のテスト名に日本語(マルチバイト文字)を使っているため、macos-latest ランナーで `bats` が `unknown test name` で全テストを実行できずに失敗する問題が発生した。当初 UTF-8 ロケール未設定が原因と推測して `LANG`/`LC_ALL=en_US.UTF-8` を設定したが解消せず、CI 上でデバッグステップを追加して調査した結果、真因はロケールではなく **bash のバージョン**だった。macOS ランナーの `/bin/bash` は Apple 標準搭載の bash 3.2(2007年、GPLv2最終版でマルチバイト文字列処理に非互換がある)で、`bats` 自身の `#!/usr/bin/env bash` シェバンがこれを解決してしまっていた(開発機はシェル起動時に Homebrew の新しい bash が PATH 優先で解決されるため気づけなかった)→ [Mitigation] macOS ジョブで `brew install bash` した上で Homebrew の bin ディレクトリを `$GITHUB_PATH` に追加し、以降のステップで `env bash` が新しい bash を解決するようにした(UTF-8 ロケール設定自体は無害なので残した)

## Migration Plan

新規追加のみで既存の実行時挙動(`bin/bw-quickaccess` の動作)に変更はない。ロールバックが必要な場合は `.github/workflows/ci.yml` と `test/` ディレクトリを削除するだけで元の状態に戻せる。

## Open Questions

(実装時に解消済み。fixtures は Bitwarden テストアカウントが無いため `bw list items`/`bw get item` の実際のフィールド構成を参考に手作文し、`test/fixtures/` にコミットした。shellcheck の除外ルールは Decision 6 の通り確定した)

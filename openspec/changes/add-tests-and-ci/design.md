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

### 6. shellcheck の除外ルール: `.shellcheckrc` + ピンポイントのインライン disable

**決定**: リポジトリ直下に `.shellcheckrc`(`shell=bash`、`disable=SC1091,SC2034,SC2016,SC2329`)を追加する。`lib/*.sh` と `test/helpers/stub.bash`・`test/lib/*.bats` は他ファイルから source される前提の設計であり、単体で shellcheck にかけると shebang 不在(SC2148。`shell=bash` で解消)・動的パスによる追跡不可(SC1091)・他ファイルで消費される定数の誤検知(SC2034)・意図的なシングルクォート(`bwqa_test_stub_cmd` の遅延展開設計、SC2016)・関数スタブが他ファイル側から間接的に呼ばれることによる「未使用関数」誤検知(SC2329)が構造的に発生することを実装時に確認した。これらはプロジェクトの sourcing アーキテクチャ・テスト用モック設計に起因する既知の誤検知としてグローバルに無効化する。

一方、`lib/fields.sh` の `bwqa_run_field_screen`/`bwqa_copy_field_internal` にあった `export BW_SESSION`/`BWQA_ITEM_ID` を subshell 内で行うパターン(fzf の `execute-silent` 経由で再起動される子プロセスへ環境変数を継承させる意図的な設計)は SC2030/SC2031/SC2153 を発火させたが、これは特定関数に固有の設計判断であり誤検知の性質もプロジェクト全体には一般化できないため、該当箇所にのみ `# shellcheck disable=...` を付与した(`.shellcheckrc` には追加しない)。

**理由**: `.shellcheckrc` でのグローバル無効化は、sourcing アーキテクチャに起因して全ファイルで一様に発生する誤検知(SC1091/SC2034)に限定し、個別関数のロジックに起因する誤検知(SC2030/SC2031/SC2153)はインライン disable で局所化することで、将来の本当の unused variable バグ等を検知する能力をなるべく損なわないようにした。

**CI/ローカルでの呼び出し方**: `shellcheck bin/bw-quickaccess lib/*.sh test/helpers/*.bash test/lib/*.bats` のように対象ファイルを列挙して一括実行する(`-x` フラグは不要。`.shellcheckrc` の設定のみで解決する)。

## Risks / Trade-offs

- [Risk] `bw` の実出力フォーマット変更を fixtures が追従できず、テストは通るが実環境で壊れる sync 漏れが起きうる → [Mitigation] fixtures 作成時に実際の `bw list items --pretty` 等の出力を参考にする旨をテストコード内コメントで明記し、大きな bw バージョンアップ時は fixtures の見直しをタスク化する
- [Risk] PATH ダミーコマンド方式は bats の `setup`/`teardown` の実装漏れがあると他テストに影響(PATH 汚染)しうる → [Mitigation] `test/helpers/stub.bash` に PATH 復元を一元化し、各 `.bats` は個別に PATH 操作を書かない
- [Risk] macOS ランナーは起動・キューが Linux より遅く、CI のフィードバック速度が落ちる → [Mitigation] 許容(課金面の懸念がないため速度は許容範囲とする)
- [Risk] 将来 private 化した場合、macOS ランナーの消費レートが 10 倍になり無料枠を圧迫する → [Mitigation] private 化を検討する際に CI matrix 構成(macOS 除外や実行頻度の見直し)を再検討する

## Migration Plan

新規追加のみで既存の実行時挙動(`bin/bw-quickaccess` の動作)に変更はない。ロールバックが必要な場合は `.github/workflows/ci.yml` と `test/` ディレクトリを削除するだけで元の状態に戻せる。

## Open Questions

(実装時に解消済み。fixtures は Bitwarden テストアカウントが無いため `bw list items`/`bw get item` の実際のフィールド構成を参考に手作文し、`test/fixtures/` にコミットした。shellcheck の除外ルールは Decision 6 の通り確定した)

# CLAUDE.md — bitwarden-quickaccess プロジェクト

## AI ツール役割分担

このプロジェクトでは Claude Code と Antigravity CLI (agy) を役割に応じて使い分ける。

| フェーズ | ツール | 理由 |
|---------|--------|------|
| 設計・調査・探索 | **Claude Code** | 設計能力・コードベース理解が高い |
| 実装 | **Antigravity CLI** (`agy`) | Gemini クレジットを活用、Claude クレジットを節約 |
| コードレビュー・PR作成 | **Claude Code** | `/code-review` スキルを使う |
| レビュー指摘の修正 | **Antigravity CLI** (`agy`) | 実装担当が修正。Claude Code は修正内容を指示するのみ |

### 難易度の目安(タスク単位で判定)

判定は change 全体ではなく tasks.md の各タスク単位で行う。
agy をデフォルトとし、以下の条件に該当する場合のみ Claude Code が実装する。

**agy に任せる(デフォルト):**
- 既存コードにパターンがある実装(session/clipboard/keychain 連携含む)
- 既存パターンの横展開(複数 lib/*.sh への同じ変更等)
- 依存追加・設定変更・クリーンアップ
- テスト追加(既存テストパターンを参考にできる場合)

**Claude Code が実装する:**
- コードベースに前例がない新規アーキテクチャパターンの導入
- 複数コンポーネント間の複雑な状態フローの設計と実装
- プロンプトで十分な文脈を伝えきれない場合
- agy が 2 回連続で失敗した場合

### Claude Code から agy を呼び出す方法

#### 方式の選択

| 方式 | 条件 | 利点 |
|------|------|------|
| **MCP(推奨)** | `mcp__agy__antigravity_ask` ツールが利用可能 | stdout バグ回避、会話継続が自然、タイムアウト問題が少ない |
| **Bash(フォールバック)** | MCP未セットアップの環境 | セットアップ不要、どの環境でも動く |

セッション開始時に `mcp__agy__antigravity_status` が呼べるか確認し、利用可能なら MCP 方式を使う(このツール自体はクォータを消費しない)。
利用できなければ Bash 方式にフォールバックする。

> 注意: MCP ブリッジのツール名はブリッジのバージョンによって変わることがある(例: `agy_ask` → `antigravity_ask`)。
> 実際に呼び出す前に、利用可能なツール一覧(`mcp__agy__*`)を確認し、実在するツール名を使うこと。

#### MCP 方式(推奨)

MCPブリッジ経由でagyをサブエージェントとして呼び出す。

```
mcp__agy__antigravity_ask(prompt="<実装プロンプト>", workspace="<プロジェクトルート>")
```

- 新規タスク: `mcp__agy__antigravity_ask`
- 追加指示・継続: `mcp__agy__antigravity_continue`
- 診断(クォータ消費なし): `mcp__agy__antigravity_status`

`antigravity_ask` / `antigravity_continue` はデフォルト `timeout_s=180`。中規模以上のタスクでは `timeout_s=240` 以上を設定する。
また `watch=true` を指定すると agy の実行過程をブラウザでライブ確認できる(必須ではない)。

#### Bash 方式(フォールバック)

MCP が使えない環境では従来の Bash 経由で呼び出す:

```bash
GIT_TERMINAL_PROMPT=0 CI=true \
  agy --dangerously-skip-permissions --print-timeout 3m --print "<実装プロンプト>" 2>&1
```

| タスク規模 | ファイル数 | タイムアウト |
|-----------|-----------|------------|
| 小 | 1-2 | `--print-timeout 3m` |
| 中 | 3-5 | `--print-timeout 5m` |
| 大 | 6+ | `--print-timeout 8m`(さらなる分割を検討) |

`CI=true` の副作用でツールの動作が変わった場合は、当該環境変数を除外してプロンプト内ルールのみでハング予防する。

#### 共通ルール(MCP・Bash 両方式共通)

**タスク単位の分割呼び出し:**
一括委譲ではなく、OpenSpec tasks.md の各タスクを個別の agy ワンショットで実行する。
十分に小さいタスク(1ファイルの小変更)は Claude Code の判断でまとめてよい。
各タスク完了後に `git status` / `git diff` で結果を確認してから次のタスクに進む。

**プロンプトに含める内容:**
- 実装対象ファイルと変更内容(具体的に)
- 既存コードのパターン(コピーすべき書き方。例: `lib/*.sh` の関数構成、`set -euo pipefail` の踏襲)
- スコープ外の制約(「この2ファイルだけ触れ」など)
- OpenSpec の change ディレクトリへの参照(`openspec/changes/<name>/` 以下)
- **実装完了後に `git add <実装ファイル> && git commit -m "feat: <変更内容>"` でコミットすること**
- **不明点・判断に迷う点がある場合は実装せず、`[QUESTION] ...` の形式で質問を出力すること**
- **ハング予防ルール(以下の「agy プロンプトの必須ルール」セクション参照)**

実装完了後は Claude Code で `git diff HEAD` または `git log` を確認してからレビューに進む。

### agy プロンプトの必須ルール

agy へのすべてのプロンプトに、以下のルールを必ず含める:

```
制約:
- 対話的入力(y/n, パスワード等)を求めるコマンドは絶対に実行しないこと
- 必ず非対話フラグ(--yes, -y, --no-input 等)を付けること
- git push, npm publish など外部サービスへの送信は行わないこと
- 対話的入力が必要な状況に遭遇したら、実行せず [QUESTION] で報告すること
- git add は指定ファイルのみ。`git add -A` や `git add .` は禁止
```

`bw unlock` など Bitwarden CLI 自体の対話コマンドを agy に実行させないこと(マスターパスワード入力が必要なため)。動作確認が必要な場合はモック・スタブで代替するか、Claude Code もしくはユーザーに確認する。

### agy との対話ループ

agy の出力に `[QUESTION]` が含まれる場合、以下のループで対応する:

```
agy 実行 (1回目)
  ↓ 出力を解析
  ├─ [QUESTION] なし → 実装完了、レビューへ
  └─ [QUESTION] あり → 質問ごとに判定:
       ├─ Claude Code が回答できる → 回答をまとめる
       └─ 判断つかない / センシティブ → ユーザーに確認
       ↓
     agy 実行 (2回目: 回答を追加コンテキストとして渡す)
       ↓ 再度出力を解析(同じループ)
```

- MCP 方式: `antigravity_continue` で追加コンテキストを渡す
- Bash 方式: 新しい `agy --print` 呼び出しに回答を含める

**ループの上限は 3 回**。3 回で解決しない場合は Claude Code が直接実装に切り替える。

### agy 失敗時のリカバリ

agy がタイムアウト・ハング・エラーで失敗した場合:

```
失敗発生
  ↓
git status / git diff で途中成果を確認
  ├─ 成果あり → 継続指示を送る
  │   MCP: antigravity_continue で残り作業を指示
  │   Bash: --continue で再開
  └─ 成果なし → 新規セッションで別アプローチを試行
      ↓
再試行も失敗(2回連続)
  ↓
Claude Code が直接実装に切り替える
```

Bash 方式の場合、`--print-timeout` が効かないケースがある。
Bash ツールの `timeout` パラメータも併用し、agy プロセスが応答しない場合は `kill` して対処する。

### agy 委譲時の判断基準

**Claude Code がユーザーに確認すべきケース:**
- ビジネスロジックの仕様判断(「この場合どう振る舞うべきか」)
- 破壊的変更や後方互換性に関わる判断
- session token・キーチェーン連携・クリップボード等セキュリティに関わる設計判断
- 複数の妥当な選択肢がありトレードオフが明確でない場合

**Claude Code が自分で回答してよいケース:**
- 既存コードのパターンやコンベンションに関する質問
- シェル関数のインターフェース(引数・戻り値・exit code)の確認
- ファイル構成やインポート(`source`)パスの確認
- OpenSpec の design.md / specs から読み取れる仕様

---

## 開発ワークフロー

### 機能追加・バグ修正の標準フロー

```
1. 設計 & proposal コミット (Claude Code)
   /opsx:explore  → 問題を探索し設計を固める
   /opsx:propose  → change proposal を生成(proposal.md / design.md / specs / tasks.md)

   ★ /opsx:propose 完了後、Claude Code は自動的に以下を実行する(スキルの出力より優先):
     a. main ブランチにいる場合:
        git checkout -b feature/<change-name>
        git add openspec/changes/<change-name>/
        git commit -m "docs(openspec): propose <change-name>"
     b. main 以外のブランチにいる場合:
        ユーザーに「新しいブランチを作るか、現在のブランチで続けるか」を確認する

   ★ コミット後、ユーザーに proposal の確認を促す:
     「proposal をコミットしました。内容を確認して、問題なければ `/opsx:apply` で実装を開始できます。」
     → 実装開始を勝手に促さない。まずユーザーのレビューを待つ。

2. 実装 (agy タスク単位 × N)
   /opsx:apply 実行時、各タスクの難易度をタスク単位で判定する(「難易度の目安」参照)
   agy がデフォルト。Claude Code は前例のない新規パターンや、agy 2回連続失敗時のみ実装する
   tasks.md の各タスクを個別の agy ワンショットで実行
   MCP: mcp__agy__antigravity_ask(prompt="...", workspace="<プロジェクトルート>")
   Bash: GIT_TERMINAL_PROMPT=0 CI=true agy --dangerously-skip-permissions --print-timeout 3m --print "..." 2>&1
   → 各タスク完了後に git status / git diff で確認
   → [QUESTION] があれば Claude Code が回答 or ユーザーに確認し再実行
   → 失敗時は antigravity_continue / --continue で再開 or Claude Code が引き継ぐ
   → agy が実装コードをコミットする("feat: <変更内容>")

2.5. agy 別セッションレビュー (agy 新規セッション)
   実装とは別の新規 agy セッションで diff をレビューさせる
   → 変更要約(変更ファイル一覧、問題点、設計判断、テスト充足度)を出力させる
   → Claude Code は要約をベースに最終レビューの判断材料にする
   → 変更が 1 ファイル・20 行以下の場合はこのステップを省略してよい

3. コードレビュー (Claude Code)
   /code-review(agy レビュー結果も参考にしつつ最終判断)
   → 指摘があれば、修正内容を具体的にまとめる

4. レビュー指摘の修正 (agy)
   Claude Code がレビュー結果から修正プロンプトを作成し agy に委譲する
   → 修正対象ファイル・行・具体的な変更内容を指示する
   → agy が修正コードをコミットする("fix: <修正内容>")
   → 指摘がなければこのステップはスキップ

5. PR 作成 & OpenSpec アーカイブ (Claude Code)
   /opsx:archive  → change をアーカイブ(delta spec sync を含む)
   gh pr create
   → アーカイブと spec sync のコミットを PR に含める

6. CI 確認 (Claude Code)
   `.github/workflows/ci.yml` により push/PR ごとに CI(macos-latest / ubuntu-latest で
   構文チェック・shellcheck・bats テスト)が実行される。
   PR 作成後は以下のコマンドで結果を待ち、失敗があれば修正してプッシュする:
   gh pr checks --watch
```

### ドキュメント更新の原則

**設計・実装・レビューのすべての場面で常に検討すること。**

- 仕様変更・機能追加があれば `openspec/specs/` の対応する spec.md を更新する
- README.md の使い方・必要なもの・スコープ外の記述に影響する変更があれば反映する
- CLAUDE.md 自体のワークフローや規約が変わったときは即座に更新する
- agy への実装プロンプトにも「関連ドキュメントの更新が必要か検討すること」を明示する

### pre-commit / CI ルール

CI ワークフロー(`.github/workflows/ci.yml`)は設定済み。push/PR ごとに
macos-latest / ubuntu-latest 上で構文チェック(`bash -n`)・`shellcheck`・
`bats test/lib/*.bats` が実行される。pre-commit フックは現時点で未設定。
今後 pre-commit フックを追加した場合も以下を厳守する:

**pre-commit のスキップ禁止**
`--no-verify` や `SKIP=...` による pre-commit フックのスキップは一切行わない。
フックが失敗した場合は、スキップせず根本原因を修正してから再コミットする。

**GitHub Actions の失敗は必ず是正する**
PR 作成後は必ず次のコマンドで CI 結果を待ち、失敗があれば修正してプッシュすること:

```bash
gh pr checks --watch   # 成功: exit 0 / 失敗: exit 非0
```

失敗した場合は `gh pr checks` でどのジョブが落ちたかを確認し、修正 → コミット → プッシュ → 再度 watch のサイクルで是正する。

### OpenSpec チートシート

| スキル | 用途 |
|--------|------|
| `/opsx:explore` | アイデア・問題を探索する(実装しない) |
| `/opsx:propose` | change proposal を一括生成 |
| `/opsx:apply` | tasks.md のタスクを順に実装(Claude Code が担当する場合) |
| `/opsx:sync` | delta spec を main spec にマージ |
| `/opsx:archive` | 完了した change をアーカイブ(PR 作成前に実施) |

### ブランチ命名規則

```
feature/<change-name>   # 機能追加(OpenSpec change 名と一致させる)
fix/<change-name>       # バグ修正
```

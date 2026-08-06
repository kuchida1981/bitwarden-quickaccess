## Context

新規リポジトリに、`bw`(Bitwarden CLI)・`jq`・`fzf` を組み合わせたターミナル向けクイックアクセスツールを実装する。既存コードはなく、制約は探索セッションで確認済みの以下の事実に基づく:

- Bitwarden は zero-knowledge 設計のため、API key(client_credentials)は `bw login` の非対話化にしか使えず、`bw unlock`(vault 復号)には常にマスターパスワードが必要
- Bitwarden デスクトップアプリの `bitwarden://` URI スキームは特定アイテムへの直接ナビゲーションに対応していない
- 対応環境は macOS と、デスクトップ GUI が動く Linux(OS keychain 利用のため)

## Goals / Non-Goals

**Goals:**
- `fzf` によるインクリメンタルサーチから、フィールド選択・クリップボードコピーまでを一連の操作で完結させる
- `bw unlock` の session token を OS keychain にキャッシュし、TTL 内は再認証を求めない
- 機密情報(パスワード・TOTP・session token)を標準出力やプロセス引数に残さない
- 依存ツール欠如時は明示的にエラー終了し、インストール方法を案内する

**Non-Goals:**
- Bitwarden デスクトップアプリとの deep link 連携
- クリップボードの自動クリア
- Linux ヘッドレス/SSH 環境のサポート
- Windows サポート

## Decisions

### 1. 実装形態: 単一の bash スクリプト
`bw`/`jq`/`fzf`/OS 標準コマンドを呼び出す薄いラッパーであり、状態(直前アイテム ID、session token の TTL)もローカルファイルと OS keychain に閉じるため、Go や Rust でバイナリ化するほどの複雑さはない。POSIX sh ではなく bash を選ぶのは、連想配列・`[[ ]]`・`local` 等を使って可読性を保つため。エントリポイントは 1 ファイル(`bin/bw-quickaccess`)とし、機能ごとに関数分割する。

代替案として検討した Go 単一バイナリは、クロスコンパイルの利便性はあるが、外部コマンド(`bw`/`fzf`/クリップボードコマンド/keychain コマンド)への `exec` が結局大半を占めるため、shell に対する優位性が薄いと判断し採用しない。

### 2. 画面遷移はシェル側の制御ループで管理し、fzf プロセスは都度起動する
「検索画面 → フィールド選択画面 → (Esc で検索へ / 連続コピー)」という状態遷移を、fzf の `become()` アクションでプロセスを繋ぐ方式も検討したが、`become()` は比較的新しい fzf バージョン(0.42+)でしか使えず、環境依存が増える。代わりに、bash 側で `while` ループを持ち、fzf の終了コード・出力(選択されたキー種別を含む)を見て次にどの画面を起動するかを判定するシンプルな構成にする。

```
┌─────────────┐   選択      ┌───────────────────┐
│ 検索画面(fzf)│ ─────────▶ │ フィールド選択画面(fzf) │
│ 全アイテム一覧 │            │ ・Enter → password  │
└─────────────┘            │ ・ctrl-u → username │
     ▲                      │ ・ctrl-t → totp     │
     │        Esc            │ (コピー後も画面継続)  │
     └──────────────────────┴───────────────────┘
```

フィールド選択画面自体は `--bind` で各フィールドのコピーを `execute-silent` にバインドし、fzf を終了させずに繰り返しコピーできるようにする。Esc または `q` バインドで検索画面へのループに戻す。

### 3. 直前アイテムの記憶は平文キャッシュファイル
`~/.cache/bw-quickaccess/last-item-id` にアイテムの UUID のみを保存する。UUID 自体は vault を復号しないと意味を持たない識別子であり機密性はないため、暗号化やパーミッション強化は不要と判断。起動時にこのファイルがあれば検索画面をスキップし、該当アイテムのフィールド選択画面から始める。

### 4. session token のキャッシュと TTL 検証は「発行時刻ベース」+「実利用時の遅延検証」の併用
`bw unlock --raw` で得た token を OS keychain(macOS: `security add-generic-password`、Linux: `secret-tool store`)に保存すると同時に、発行時刻を `~/.cache/bw-quickaccess/session-issued-at` に記録する。起動時にまず発行時刻からの経過時間を TTL(デフォルト 15 分、環境変数で上書き可能)と比較し、超えていれば無条件に再 `unlock` する。TTL 内であっても、`bw` コマンドがその session を「invalid」として拒否した場合(手動で `bw lock` された等)はその場で再 `unlock` にフォールバックする。TTL 判定のみに頼らず実利用時のエラーも見るのは、`bw lock`/`bw logout` がツールの外側で実行されるケースに対応するため。

### 5. 機密情報の受け渡しはコマンド引数を避ける
`bw get password/username/totp` の出力は、クリップボードコピーコマンド(`pbcopy`/`xclip`/`xsel`/`wl-copy`)に必ずパイプ(stdin)で渡す。session token も環境変数 `BW_SESSION` 経由で `bw` に渡し、シェルのコマンドライン引数には一切載せない(`ps` 等でのプロセス一覧からの漏洩を防ぐ)。

### 6. クリップボードコマンドの選択はランタイム検出
`uname` で OS を判定し、macOS なら `pbcopy` 固定。Linux は `$WAYLAND_DISPLAY` が設定されていれば `wl-copy`、`$DISPLAY` があれば `xclip`(なければ `xsel`)を優先する。該当コマンドが未導入ならインストール案内(`brew install` / `apt install` 等はディストリビューション検出まではせず、パッケージ名のみ案内)を出してエラー終了する。

## Risks / Trade-offs

- [Risk] session token を keychain に長時間キャッシュすることは、端末乗っ取り時の被害時間を延ばす → Mitigation: デフォルト TTL を短め(15分)にし、明示的にキャッシュを破棄する `bw-quickaccess lock` サブコマンドを用意する
- [Risk] フィールド選択画面で `execute-silent` を使う場合、実行するシェルコマンド文字列の組み立てにインジェクションの余地が生まれやすい → Mitigation: アイテム ID 等の可変値は環境変数経由でサブプロセスに渡し、シェル文字列への直接埋め込みを避ける
- [Risk] Linux でキーリングデーモン(GNOME Keyring/KWallet)が起動していない環境では `secret-tool store/lookup` が失敗する → Mitigation: 起動時に `secret-tool` の疎通を確認し、失敗時はキャッシュなし(毎回 unlock)にフォールバックする旨を明示メッセージで案内する(サイレントフォールバックはしない)
- [Risk] fzf のバージョンによって `--bind` の構文(`execute-silent` の対応など)に差がある → Mitigation: preflight で `fzf --version` を確認し、最低バージョンを満たさない場合はエラー終了してアップグレードを案内する

## Open Questions

- 配布方法(Homebrew tap 化するか、リポジトリを clone して手動で PATH に通す運用に留めるか)は未決定。初期リリースでは後者(手動運用)を前提として進めてよいか
- session TTL のデフォルト値(15分と仮置き)は実運用しながら調整する前提でよいか
- fzf の最低バージョン要件(`execute-silent` が安定して使える版)は実装時に `fzf --help`/changelog を確認して確定する

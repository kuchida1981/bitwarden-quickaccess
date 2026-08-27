## Context

`app/src-tauri/src/main.rs:54-80` の `fix_path_env()` は `main()` の最初(`tauri::Builder`構築より前、つまりtokio非同期ランタイムが存在しない時点)で同期的に1回だけ呼ばれる。現状の実装:

```rust
fn fix_path_env() {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let result = std::process::Command::new(shell)
            .args(["-l", "-c", &format!("echo -n {PATH_MARKER}; printenv PATH")])
            .output();
        let _ = tx.send(result);
    });

    let Ok(Ok(output)) = rx.recv_timeout(Duration::from_secs(3)) else {
        return; // タイムアウト時、workerスレッドと子プロセスは放置される
    };
    // ...
    unsafe { std::env::set_var("PATH", path); } // SAFETYコメントは「他スレッドが環境変数を
                                                 // 触っていない」ことを前提にしているが、
                                                 // コードでは保証されていない
}
```

`app/src-tauri/src/backend/process.rs` には`bw serve`常駐監視用のtokioベースkillパターン(`spawn_supervised_with_command`、`kill_on_drop`、`oneshot`)が既にあるが、これは`tauri::async_runtime`が起動した後でのみ使えるパターンであり、`fix_path_env()`が呼ばれる時点(ランタイム起動前)には適用できない。

## Goals / Non-Goals

**Goals:**
- タイムアウト時に子プロセス(ログインシェル)とworkerスレッドが確実に後始末される(リークしない)こと。
- `unsafe { std::env::set_var(...) }`実行時点で他スレッドが環境変数を操作し得ないことを、コメントの主張ではなく構造的に保証すること。
- poll/kill/reapの中核ロジックを、実プロセスを使った高速なユニットテストで検証可能にすること。

**Non-Goals:**
- タイムアウト値(3秒)自体の変更。
- ログインシェルが起動する孫プロセス(rcファイルが起動しうるバックグラウンドジョブ等)のケア。直接の子プロセスをkillすれば十分とし、孫プロセスの追跡は行わない(現状の実装でも同様に未対応であり、今回悪化させない範囲に留める)。
- tokio/async化。呼び出し時点でランタイムが存在しないため同期実装のまま。

## Decisions

### 1. 追加スレッド+channelをやめ、mainスレッドのみの同期ポーリングにする

**採用案**: `Command::spawn()`で子プロセスを起動し、`child.try_wait()`を短い間隔(50ms)でポーリングする。デッドライン(3秒)以内に終了すれば標準出力を読んでPATHを抽出し、超過したら`child.kill()` → `child.wait()`で確実にkill・reapしてから抜ける。

**却下した代替案**: workerスレッド構成を維持しつつ、子プロセスのハンドルを`Arc<Mutex<Option<Child>>>`等でmainスレッドと共有し、タイムアウト時にmainスレッドからkillしてから`.join()`する案。
- 却下理由: スレッド間の共有可変状態が増え、実装・レビューの複雑さが増す割に、得られる効果(「他スレッドが環境変数に触れない」という保証)は「スレッドを最初から作らない」案でも同等以上に得られる。並行性の軸自体をなくす方が、SAFETYの議論がそもそも不要になり筋が良い。

ポーリング間隔は50msとする(3秒÷50ms=最大60回の`try_wait`呼び出し、CPU負荷は無視できる水準)。テスト側ではこの間隔をパラメータとして注入し、5ms程度まで短縮して高速に検証する。

### 2. poll/kill/reapの中核ロジックをテスト用に分離する

`process.rs`の`spawn_supervised_with_command`が`Command`を引数として受け取ることでテスト時に`sh -c "sleep 5"`等を注入できるようにしているのと同じパターンを踏襲する。

`set_var`を呼ばない、タイムアウト値とポーリング間隔を引数に取る関数(例: `run_shell_and_capture_stdout(command: Command, timeout: Duration, poll_interval: Duration) -> Option<String>`)に処理を切り出し、`fix_path_env()`自体は「SHELL環境変数を読む→上記関数を呼ぶ→結果があれば`set_var`する」という薄いラッパーに留める。`set_var`を伴う最終ステップ自体は(既存同様)実プロセス環境を汚すためユニットテスト対象外とする。

### 3. 子プロセス終了後にstdoutを読む(パイプデッドロックの回避)

`Command::output()`は内部で標準出力・標準エラーを別スレッドで並行に読み取ることでパイプバッファ枯渇によるデッドロックを回避している。`spawn()`+手動`try_wait()`ポーリングに置き換えると、ポーリング中は誰もパイプを読まないため、シェルの出力(rcファイルのログ等)がOSパイプバッファ(macOSで通常64KB)を超えて書き込まれた場合、子プロセスが`write()`でブロックし、`try_wait()`が`Exited`を返さないまま3秒のデッドラインに達する可能性がある。

この場合でも、デッドライン超過時には`kill()`が実行されるため**ハングし続けることはなく**、単に「PATH取得に失敗しPATHが補正されないまま起動する」という既存のフォールバック動作に収束する(現状想定している出力は`echo -n {marker}; printenv PATH`のみで数十バイト程度であり、通常のシェル起動では64KBに達することは考えにくい)。子プロセスが正常終了した後は、`child.stdout`から`read_to_string`で読み取る(終了済みプロセスの書き込み端は閉じているため`read_to_string`はブロックせずEOFに到達する)。

## Risks / Trade-offs

- **[Risk] ポーリング間隔(50ms)により、成功時の応答が最大50ms遅延しうる** → 現状のスレッド+channel方式でも実質的な応答時間は同程度であり、体感できるほどの劣化ではないため許容する。
- **[Risk] 孫プロセスが残る可能性(Non-Goalsで明記)** → 現状の実装でも同じ問題を抱えており、今回のスコープでは悪化させない(直接の子プロセスは確実にkillする)ことをもって許容する。
- **[Risk] テストで実際にプロセスをspawn/killするため、CI環境(macos-latest)でのプロセス生成コストや稀なタイミング差異による不安定化の可能性** → `process.rs`の既存テストが同様のパターン(`sh -c "sleep 5"`)で安定稼働している実績があり、同じ方針を踏襲することでリスクを抑える。

## Migration Plan

内部実装のみの変更であり、外部インターフェース・データ形式の変更を伴わないため、特別な移行手順は不要。ロールバックは通常のリバートで対応可能。

macOS実機での動作確認(通常起動、ログインシェルが意図的に遅延/ハングするケース)をリリース前の確認事項とする(tasks.mdに手順を記載)。

## Open Questions

なし。

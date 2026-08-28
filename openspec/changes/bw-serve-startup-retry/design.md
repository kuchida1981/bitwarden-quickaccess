## Context

現在の起動フローは以下の通り(`main.rs::start_backend` → `process.rs`):

```
pick_free_port()                         ← 1回だけ
  → spawn_supervised(port, state)
       → build_bw_serve_command(port).spawn()   ← OSレベルのexec失敗のみErrで返る
       → tokio::spawn(監視タスク)                ← 呼び出し元から切り離される
            select! {
              child.wait()  → state.set_error("bw serveプロセスが予期せず終了しました…")
              kill_rx       → 明示shutdown、stateは変更しない
            }
```

`pick_free_port()` は `TcpListener::bind` で一時的にポートを確保してすぐ解放するため、確保から `bw serve` の実起動までの間(TOCTOU)に他プロセスがポートを奪う可能性がある。この場合 `bw serve` は起動直後にbind失敗で異常終了するが、それは `command.spawn()` のErrとしては現れず、監視タスクの `child.wait()` 分岐(「予期せず終了」)としてのみ観測される。しかも監視タスクは`tokio::spawn`で切り離されているため、呼び出し元(`start_backend`)はこの失敗を知る術がなく、リトライしようがない。

## Goals / Non-Goals

**Goals:**
- `bw serve` が起動直後に異常終了した場合、原因を問わずポートを再取得して自動リトライする
- リトライ上限に達した場合は現行と同じ手段(`state.set_error()`)でユーザーに通知する
- 安定稼働後の「予期せぬ終了検知」という既存の挙動・メッセージはそのまま維持する

**Non-Goals:**
- ポートbind失敗の原因(EADDRINUSEかどうか等)を特定すること(stderrパース等は行わない)
- `--hostname` の変更(別change `bw-serve-hostname-ipv4` で対応済み)
- 構造化ロギング基盤の導入(issue #84 は別スコープ、今回は `eprintln!` を踏襲)
- 起動確認(readinessポーリング)が完了した後の再起動・自動復旧(既存通りエラー画面を出してユーザーに再起動を促す)

## Decisions

### 1. 「起動失敗」の判定: readinessポーリングが完了する前の早期終了で判定する
`sync_initial_status()` の起動確認ポーリング(200ms間隔・最大10回=最大2秒)が完了する前に子プロセスが終了したら、原因を問わず「起動失敗」とみなす。固定タイマーを新設するのではなく、既存の起動確認ウィンドウをそのまま流用する(詳細は決定3)。stderrをパースしてEADDRINUSE等を検出する方式は、`bw` のバージョン・OSロケールによってメッセージが変わり壊れやすいため採用しない。

### 2. リトライ回数: 最大3回(初回+2リトライ)
`MAX_STARTUP_ATTEMPTS = 3`。3回とも起動直後に異常終了した場合のみ、現行と同じ `state.set_error()` 経路でエラー状態にする。毎回 `pick_free_port()` からやり直し、同じポートは再利用しない。

### 3. アーキテクチャ: 既存の readiness ポーリングと「早期終了」を競合(race)させる

**却下した案**: 起動直後に固定のグレース期間(2秒)分だけ同期的に待ってから呼び出し元に返す設計は、リトライが発生しない通常ケースでも起動のたびに必ず2秒の遅延を乗せてしまうため採用しない。

**採用する案**: `main.rs` は既に `sync_initial_status()` で `/status` を200ms間隔・最大10回(=2秒)ポーリングして起動確認を行っている。この既存の待機時間をそのまま「早期終了かどうかを判定する猶予期間」として再利用し、`tokio::select!` で「readinessポーリングの完了」と「子プロセスの早期終了」を競わせる。どちらが起きてもすぐに反応できるため、通常ケース(競合なし)では追加の遅延が一切発生しない。

```rust
// process.rs
// 監視タスクを「起動確認中(exit=リトライ対象)」と「安定稼働中(exit=致命的)」の
// 2フェーズに分離する。呼び出し元が confirm を送るまでは、exitしても
// state には一切触れず、oneshotで呼び出し元に通知するだけ。
pub(crate) struct StartupHandles {
    pub process_handle: ProcessHandle,
    pub monitor: JoinHandle<()>,
    pub exited: oneshot::Receiver<()>,   // 起動確認中にexitした場合に一度だけ発火
    pub confirm: oneshot::Sender<()>,    // 呼び出し元が起動確認成功を伝える(以後exitは致命的に扱う)
}

pub(crate) fn spawn_supervised_for_startup(
    port: u16,
    state: AppState,
) -> io::Result<StartupHandles> { ... }
```

監視タスクの内部構造(概念):
```
select! {
    _ = child.wait() => {
        exited_tx.send(());  // 起動確認中の早期終了。state には触れない
    }
    _ = confirm_rx => {
        // 呼び出し元がreadiness確認済み。以後は現行通りの「予期せぬ終了」監視に切り替える
        select! {
            _ = child.wait() => { state.set_error("bw serveプロセスが予期せず終了しました…") }
            _ = kill_rx       => { /* 明示shutdown、stateは変更しない */ }
        }
    }
}
```

`main.rs::start_backend` 側の1試行分の流れ:
```
port = pick_free_port()
handles = process::spawn_supervised_for_startup(port, state.clone())?
client = BwServeClient::new(port)

select! {
    _ = sync_initial_status(&client, &state) => {
        // 成功/タイムアウトいずれにせよ readiness ポーリングが完了した
        // (タイムアウト側は sync_initial_status が現行通り state.set_error() 済み)
        handles.confirm.send(());  // 以後の監視は「予期せぬ終了=致命的」に切り替える
        // ManagedProcess へ登録し、port を state.set_port(port) する
        return Success;
    }
    _ = handles.exited => {
        // readiness確認が完了する前にexit = 起動失敗。state には未だ何も書いていない
        eprintln!("bw serve がport {port} での起動確認中に終了しました(試行 {attempt}/{MAX_STARTUP_ATTEMPTS})。ポートを再取得してリトライします。");
        continue;  // 次の試行(新しいポートで再spawn)
    }
}
```

これを `MAX_STARTUP_ATTEMPTS`(=3)回繰り返す。ポイント:
- `sync_initial_status` 自体は変更しない(タイムアウト時に `state.set_error("起動確認がタイムアウトしました")` を呼ぶ現行の責務をそのまま持つ)。この分岐が勝った場合はリトライしない(=「早期exit以外は理由を問わずリトライしない」という合意方針通り)
- `exited` 分岐が勝った場合のみリトライする。この分岐が勝つ限り `state` は一切変更されないため、エラー画面がちらつく(flicker)ことはない
- 全試行が早期終了で尽きた場合のみ、`start_backend` 側で `state.set_error(format!("bw serve の起動に{MAX_STARTUP_ATTEMPTS}回失敗しました。アプリを再起動してください。"))` を呼ぶ
- 既存の `spawn_supervised` / `spawn_supervised_with_command`(および既存テスト `crash_updates_state_to_disconnected` 等)はそのまま温存し、`spawn_supervised_for_startup` は同じ構築パーツ(`build_bw_serve_command`・`kill_on_drop` 等)を再利用しつつ2フェーズ監視のためだけに新設する

## Risks / Trade-offs

- [Risk] readinessポーリングの2秒ウィンドウを過ぎてから発生した遅延クラッシュは今回のリトライ対象外 → Mitigation: 現行通り「予期せぬ終了」として `state.set_error()` される(挙動は変わらないだけで悪化はしない)
- [Risk] 起動が遅い環境(初回起動時のディスクI/O等)で正常起動が2秒を超える場合、readinessポーリング自体がタイムアウトしてエラーになる(exitはしていないためリトライはされない) → Mitigation: これは今回の変更で新たに生まれる挙動ではなく現行のまま。リトライ対象は「早期終了」に限定するという合意方針の範囲内
- [Risk] 最大3回リトライすると、競合が連続した場合は起動確認ウィンドウ(最大2秒)×3回分、体感の起動が遅くなりうる → Mitigation: 通常ケース(競合なし)では1回目の試行がreadinessポーリング成功で即座に完了するため追加の遅延は発生しない。連続競合が起きること自体が稀

## Open Questions

なし(リトライ回数・アーキテクチャは本designで確定)

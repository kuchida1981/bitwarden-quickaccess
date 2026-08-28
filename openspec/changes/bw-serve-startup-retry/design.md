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

**実装時の補足**: `main.rs::start_backend` のリトライループ本体は `acquire_backend_process(state, build_command, readiness_check, register_process)` という汎用関数として切り出した。`build_command: FnMut(u16) -> Command` と `readiness_check: FnMut(u16) -> impl Future<Output = ()>` を注入可能にすることで、実際の `bw` CLIやHTTPサーバなしにリトライ分岐(早期終了→リトライ / 起動確認完了→成功/上限到達→エラー)を単体テストできるようにしている。本番経路では `build_command` に `process::build_bw_serve_command` を、`readiness_check` に `sync_initial_status(&BwServeClient::new(port), &state)` をラップしたクロージャを渡す。`BwServeClient::new` は `http://localhost:{port}` を用いる(hostname解決に依存する。この点は別change `bw-serve-hostname-ipv4` の対象外であり、既存の挙動のまま)ため、テストでは実HTTP通信を避け `readiness_check` 自体をダミーの遅延完了に差し替えている。

**実装レビューでの修正**: 当初、`ManagedProcess`(トレイの終了処理が参照する `ProcessHandle` の置き場)への登録はリトライループ全体が成功で終わった後にまとめて行っていたが、これだと最大3試行×起動確認ウィンドウ分の間にアプリが終了された場合、起動済みの子プロセスが `ManagedProcess` に登録されておらずkillされない(孤児化する)問題があった(agyおよびClaude Codeのコードレビューで指摘)。`register_process: FnMut(ProcessHandle)` を追加し、各試行でspawnした直後・起動確認を待つ前に呼び出す形に修正した。試行が失敗して次のポートで再spawnする際は、新しいハンドルで上書き登録する(古いハンドルは対応する監視タスクが既に終了しているため、破棄してよい)。

`ProcessHandle` の drop は `kill_tx`(oneshot Sender)を閉じることになり、これは監視タスクにとって明示的な `shutdown()` 呼び出しと区別がつかない(既存の `spawn_supervised`/`ProcessHandle` の設計から踏襲した挙動)。そのため `register_process` に渡すクロージャでハンドルを即座にdropしてはならない(テストでこの点を誤り、生存しているはずの子プロセスを早期killしてしまうバグを一度作り込んだ)。

同じレビューで、`state.set_port(port)` を `acquire_backend_process` の戻り値を受け取った後(=リトライループ全体の成功後)に呼んでいたため、`readiness_check`(実運用では `sync_initial_status`)が完了時に内部で呼ぶ `state.set_locked()`/`set_unlocked()` より後になってしまう不整合も見つかった。「ロック状態はセット済みだが `state.port()` は `None`」という一瞬のウィンドウが生まれ、その間に `commands::client_for` が「バックエンドサービスの準備がまだできていません」を誤って返しうる。`state.set_port(port)` は各試行でspawn成功直後(`register_process` の直後、`readiness_check` を待つ前)に呼ぶよう修正し、この順序を固定する回帰テスト(`port_is_recorded_before_readiness_check_runs`)を追加した。

**さらなるレビューでの修正(2件)**:
1. `state.set_port(port)` を試行のたびに呼ぶようにした副作用として、全試行が早期終了で尽きた場合に `state.port()` が最後の(死んだ)試行のポート番号を保持したまま残ってしまう問題が見つかった。個別に `clear_port()` を呼ぶのではなく、`AppState::set_error()` 自体が `port` を同時にクリアするよう修正した(エラー状態になった時点でその `port` はどのみち無効なので、`set_error` を呼ぶ全箇所——起動失敗・プロセスクラッシュ・ログイン未実施等——で一貫して安全になる)。
2. `process.rs` の `spawn_supervised_with_command` と `spawn_supervised_for_startup_with_command`(confirm受信後)で、「予期せぬ終了→`state.set_error()`」の監視ロジックが重複していた。共通の `async fn supervise_until_exit(child, state, kill_rx)` に切り出し、両方から呼ぶように統合した。

**4回目のレビューでの修正**: リトライ機構の導入により `start_backend` が `spawn_supervised_for_startup_with_command` 経由の起動しか使わなくなった結果、単発起動用の `spawn_supervised(port, state)` がどこからも呼ばれないdead codeになっていた(`pub` だが実質的に呼び出し元がない)。この関数と、それが内部で使っていた `spawn_supervised_with_command`(テストからしか呼ばれておらずこちらもdead code化していた)を削除し、共有ロジック `supervise_until_exit` を対象テストが直接呼ぶ形に整理した。あわせて `crash_updates_state_to_disconnected` は `spawn_supervised_for_startup_with_command` 経由の `crash_after_confirm_updates_state_to_disconnected` と同じシナリオを検証する重複テストだったため、`supervise_until_exit` を直接検証する形に統合した。

**5回目のレビューでの修正(2件)**:
1. 同様の理由で `spawn_supervised_for_startup(port, state)`(`_with_command` ではない方のラッパー)もどこからも呼ばれないdead codeになっていたため削除した。
2. より重要な指摘として、`acquire_backend_process` のリトライ判定が「起動確認完了前のexit」を理由を問わず一律リトライしていたため、アプリ終了処理(`RunEvent::Exit` → `ProcessHandle::shutdown()`)が発行された場合と純粋なクラッシュを区別できていなかった。リトライ中(起動確認ウィンドウ内)にユーザーがアプリを終了すると、`ManagedProcess` は既に空になっている(`RunEvent::Exit` のハンドラが `guard.take()` 済み)にもかかわらず、ループが「クラッシュした」と誤認して新しいポートで `bw serve` を再spawnし、その新プロセスがどこにも登録されず孤児化しうる欠陥があった。`process::StartupExit`(`Crashed` / `ShutdownRequested`)を導入して `exited` チャンネルに理由を持たせ、`ShutdownRequested` の場合はリトライせず即座に処理を終了するよう修正した。

## Risks / Trade-offs

- [Risk] readinessポーリングの2秒ウィンドウを過ぎてから発生した遅延クラッシュは今回のリトライ対象外 → Mitigation: 現行通り「予期せぬ終了」として `state.set_error()` される(挙動は変わらないだけで悪化はしない)
- [Risk] 起動が遅い環境(初回起動時のディスクI/O等)で正常起動が2秒を超える場合、readinessポーリング自体がタイムアウトしてエラーになる(exitはしていないためリトライはされない) → Mitigation: これは今回の変更で新たに生まれる挙動ではなく現行のまま。リトライ対象は「早期終了」に限定するという合意方針の範囲内
- [Risk] 最大3回リトライすると、競合が連続した場合は起動確認ウィンドウ(最大2秒)×3回分、体感の起動が遅くなりうる → Mitigation: 通常ケース(競合なし)では1回目の試行がreadinessポーリング成功で即座に完了するため追加の遅延は発生しない。連続競合が起きること自体が稀

## Open Questions

なし(リトライ回数・アーキテクチャは本designで確定)

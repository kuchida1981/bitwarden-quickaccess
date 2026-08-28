use std::io;
use std::net::TcpListener;
use std::process::Stdio;

use tokio::process::{Child, Command};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

use super::state::AppState;

/// `bw serve` が起動確認(readinessポーリング)完了前に終了した場合の最大試行回数(初回含む)。
pub const MAX_STARTUP_ATTEMPTS: u32 = 3;

/// OSに空きポートを割り当てさせ、そのポート番号を返す(`bw serve --port` に渡す用途)。
/// 一時的にbindしてすぐ解放するだけなので、割り当てから実際の起動までの間に
/// 他プロセスに奪われる可能性はゼロではないが、固定ポートの衝突回避が目的であり
/// セキュリティ境界としては扱わない(design.md 参照)。
pub fn pick_free_port() -> io::Result<u16> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    Ok(listener.local_addr()?.port())
}

/// `bw serve` の起動用コマンドを組み立てる。
pub fn build_bw_serve_command(port: u16) -> Command {
    let mut cmd = Command::new("bw");
    cmd.args(["serve", "--hostname", "localhost", "--port", &port.to_string()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    cmd
}

/// `bw serve` を子プロセスとして起動する。標準入出力は継承せず破棄する。
pub fn spawn_bw_serve(port: u16) -> io::Result<Child> {
    build_bw_serve_command(port).spawn()
}

/// 起動中の `bw serve` プロセスへのハンドル。`shutdown()` で明示的に終了できる。
pub struct ProcessHandle {
    kill_tx: Option<oneshot::Sender<()>>,
}

impl ProcessHandle {
    /// アプリ終了処理から呼び出し、子プロセスを確実に終了させる。
    pub fn shutdown(&mut self) {
        if let Some(tx) = self.kill_tx.take() {
            let _ = tx.send(());
        }
    }
}

/// 子プロセスの終了を待ち、「予期せぬ終了」であれば `state` にエラーとして記録する。
/// `kill_rx` 経由の明示的なshutdownの場合は `state` を変更しない。
/// `spawn_supervised_for_startup_with_command`(confirm受信後)から使われる。
async fn supervise_until_exit(mut child: Child, state: AppState, mut kill_rx: oneshot::Receiver<()>) {
    tokio::select! {
        _ = child.wait() => {
            state.set_error("bw serve プロセスが予期せず終了しました。アプリを再起動してください。");
        }
        _ = &mut kill_rx => {
            let _ = child.start_kill();
            let _ = child.wait().await;
        }
    }
}

/// 起動確認中の監視ハンドル一式。呼び出し元は起動確認(readinessポーリング)が完了したら
/// `confirm` を送信しなければならない。送信前に `bw serve` が終了した場合は `exited` が
/// 一度だけ発火し、`state` は一切変更されない(呼び出し元がリトライするかどうかを判断できるように)。
pub struct StartupHandles {
    pub process_handle: ProcessHandle,
    pub monitor: JoinHandle<()>,
    pub exited: oneshot::Receiver<()>,
    pub confirm: oneshot::Sender<()>,
}

/// `bw serve` を起動する。`confirm` が送信されるまでの間に子プロセスが終了した場合は
/// `state` に触れず `exited` で通知するだけに留め(起動失敗としてリトライ可能にするため)、
/// `confirm` 送信後は「予期せぬ終了→`state.set_error()`」監視(`supervise_until_exit`)
/// に切り替わる。
pub fn spawn_supervised_for_startup(port: u16, state: AppState) -> io::Result<StartupHandles> {
    spawn_supervised_for_startup_with_command(build_bw_serve_command(port), state)
}

pub fn spawn_supervised_for_startup_with_command(
    mut command: Command,
    state: AppState,
) -> io::Result<StartupHandles> {
    let mut child = command.spawn()?;
    let (kill_tx, mut kill_rx) = oneshot::channel();
    let (exited_tx, exited_rx) = oneshot::channel();
    let (confirm_tx, mut confirm_rx) = oneshot::channel();

    let monitor = tokio::spawn(async move {
        tokio::select! {
            _ = child.wait() => {
                let _ = exited_tx.send(());
                return;
            }
            _ = &mut confirm_rx => {}
            _ = &mut kill_rx => {
                let _ = child.start_kill();
                let _ = child.wait().await;
                return;
            }
        }

        // ここに到達するのは confirm を受信した場合のみ。以後は現行同様の監視に切り替える。
        supervise_until_exit(child, state, kill_rx).await;
    });

    Ok(StartupHandles {
        process_handle: ProcessHandle {
            kill_tx: Some(kill_tx),
        },
        monitor,
        exited: exited_rx,
        confirm: confirm_tx,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::state::BackendState;
    use std::time::Duration;

    #[test]
    fn pick_free_port_returns_a_bindable_port() {
        let port = pick_free_port().expect("should find a free port");
        assert!(port > 0);

        // 割り当てられたポートに実際にbindできることを確認する。
        // cargo testはテストごとに別スレッドで並行実行されるため、他のテスト
        // (http_clientのモックサーバ等)が同じ一時ポートを一瞬先に奪うことが
        // まれにある。数回リトライして本当に恒常的な失敗のみを検出する。
        let mut last_err = None;
        for attempt in 0..5 {
            match TcpListener::bind(("127.0.0.1", port)) {
                Ok(_) => return,
                Err(err) => {
                    last_err = Some(err);
                    std::thread::sleep(Duration::from_millis(20 * (attempt + 1)));
                }
            }
        }
        panic!("port should be free again: {last_err:?}");
    }

    #[tokio::test]
    async fn supervise_until_exit_records_crash() {
        let state = AppState::new();
        state.set_unlocked();

        let mut cmd = Command::new("sh");
        cmd.args(["-c", "exit 0"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        let child = cmd.spawn().expect("failed to spawn command");
        let (_kill_tx, kill_rx) = oneshot::channel();

        supervise_until_exit(child, state.clone(), kill_rx).await;

        assert_eq!(state.backend_state(), BackendState::Disconnected);
        assert!(state.last_error().is_some());
    }

    #[tokio::test]
    async fn supervise_until_exit_explicit_shutdown_does_not_mark_disconnected() {
        let state = AppState::new();
        state.set_unlocked();

        let mut cmd = Command::new("sh");
        cmd.args(["-c", "sleep 5"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        let child = cmd.spawn().expect("failed to spawn command");
        let (kill_tx, kill_rx) = oneshot::channel();

        kill_tx.send(()).expect("kill_rx should still be alive");

        tokio::time::timeout(Duration::from_secs(3), supervise_until_exit(child, state.clone(), kill_rx))
            .await
            .expect("supervise_until_exit should finish quickly after shutdown signal");

        // 明示的なshutdownでは state は変更されない(呼び出し側がアプリ終了処理中のため)。
        assert_eq!(state.backend_state(), BackendState::Unlocked);
    }

    #[tokio::test]
    async fn exits_before_confirm_notifies_without_touching_state() {
        let state = AppState::new();
        state.set_unlocked();

        let mut cmd = Command::new("sh");
        cmd.args(["-c", "exit 1"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true);

        let handles = spawn_supervised_for_startup_with_command(cmd, state.clone())
            .expect("failed to spawn supervised command");

        handles.exited.await.expect("exited channel should fire before confirm");
        handles.monitor.await.expect("monitor task panicked");

        // confirmを送っていないので、起動失敗として state には一切触れない(呼び出し側がリトライ判断する)。
        assert_eq!(state.backend_state(), BackendState::Unlocked);
        assert!(state.last_error().is_none());
    }

    #[tokio::test]
    async fn crash_after_confirm_updates_state_to_disconnected() {
        let state = AppState::new();
        state.set_unlocked();

        let mut cmd = Command::new("sh");
        cmd.args(["-c", "sleep 0.2; exit 0"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true);

        let handles = spawn_supervised_for_startup_with_command(cmd, state.clone())
            .expect("failed to spawn supervised command");

        handles
            .confirm
            .send(())
            .expect("monitor task should still be waiting for confirm");

        handles.monitor.await.expect("monitor task panicked");

        // confirm後の終了は supervise_until_exit により予期せぬ終了として扱われる。
        assert_eq!(state.backend_state(), BackendState::Disconnected);
        assert!(state.last_error().is_some());
    }
}

use std::io;
use std::net::TcpListener;
use std::process::Stdio;

use tokio::process::{Child, Command};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

use super::state::AppState;

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

/// 指定された Command を起動し、監視タスクを立ち上げる。
pub(crate) fn spawn_supervised_with_command(
    mut command: Command,
    state: AppState,
) -> io::Result<(ProcessHandle, JoinHandle<()>)> {
    let mut child = command.spawn()?;
    let (kill_tx, kill_rx) = oneshot::channel();

    let join_handle = tokio::spawn(async move {
        tokio::select! {
            _ = child.wait() => {
                state.set_disconnected();
            }
            _ = kill_rx => {
                let _ = child.start_kill();
                let _ = child.wait().await;
            }
        }
    });

    Ok((
        ProcessHandle {
            kill_tx: Some(kill_tx),
        },
        join_handle,
    ))
}

/// `bw serve` を起動し、監視タスクを立ち上げる。
/// 子プロセスが予期せず終了した場合は `state` を `Disconnected` に更新する。
/// 戻り値の `ProcessHandle::shutdown()` を呼ぶとプロセスを終了させ、
/// この場合は監視タスクは `state` を更新しない(意図した終了のため)。
pub fn spawn_supervised(port: u16, state: AppState) -> io::Result<(ProcessHandle, JoinHandle<()>)> {
    let cmd = build_bw_serve_command(port);
    spawn_supervised_with_command(cmd, state)
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
    async fn crash_updates_state_to_disconnected() {
        let state = AppState::new();
        state.set_unlocked();

        let mut cmd = Command::new("sh");
        cmd.args(["-c", "exit 0"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true);

        let (_handle, join_handle) = spawn_supervised_with_command(cmd, state.clone())
            .expect("failed to spawn supervised command");

        join_handle.await.expect("monitor task panicked");
        assert_eq!(state.backend_state(), BackendState::Disconnected);
    }

    #[tokio::test]
    async fn explicit_shutdown_does_not_mark_disconnected() {
        let state = AppState::new();
        state.set_unlocked();

        let mut cmd = Command::new("sh");
        cmd.args(["-c", "sleep 5"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true);

        let (mut handle, join_handle) = spawn_supervised_with_command(cmd, state.clone())
            .expect("failed to spawn supervised command");

        handle.shutdown();

        tokio::time::timeout(Duration::from_secs(3), join_handle)
            .await
            .expect("monitor task should finish quickly after shutdown")
            .expect("monitor task panicked");

        // 明示的なshutdownでは state は変更されない(呼び出し側がアプリ終了処理中のため)。
        assert_eq!(state.backend_state(), BackendState::Unlocked);
    }
}

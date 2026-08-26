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

/// `bw serve` を子プロセスとして起動する。標準入出力は継承せず破棄する。
pub fn spawn_bw_serve(port: u16) -> io::Result<Child> {
    Command::new("bw")
        .args(["serve", "--hostname", "localhost", "--port", &port.to_string()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
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

/// `bw serve` を起動し、監視タスクを立ち上げる。
/// 子プロセスが予期せず終了した場合は `state` を `Disconnected` に更新する。
/// 戻り値の `ProcessHandle::shutdown()` を呼ぶとプロセスを終了させ、
/// この場合は監視タスクは `state` を更新しない(意図した終了のため)。
pub fn spawn_supervised(port: u16, state: AppState) -> io::Result<(ProcessHandle, JoinHandle<()>)> {
    let mut child = spawn_bw_serve(port)?;
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
        TcpListener::bind(("127.0.0.1", port)).expect("port should be free again");
    }

    #[tokio::test]
    async fn crash_updates_state_to_disconnected() {
        // `bw` の代わりに、少し待ってすぐ終了するダミープロセスを使う。
        let state = AppState::new();
        state.set_unlocked();

        let mut child = Command::new("sh")
            .args(["-c", "exit 0"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .expect("failed to spawn dummy process");

        let (_kill_tx, kill_rx) = oneshot::channel::<()>();
        let state_clone = state.clone();
        let join_handle = tokio::spawn(async move {
            tokio::select! {
                _ = child.wait() => {
                    state_clone.set_disconnected();
                }
                _ = kill_rx => {}
            }
        });

        join_handle.await.expect("monitor task panicked");
        assert_eq!(state.backend_state(), BackendState::Disconnected);
    }

    #[tokio::test]
    async fn explicit_shutdown_does_not_mark_disconnected() {
        let state = AppState::new();
        state.set_unlocked();

        let mut child = Command::new("sh")
            .args(["-c", "sleep 5"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .expect("failed to spawn dummy process");

        let (kill_tx, kill_rx) = oneshot::channel();
        let state_clone = state.clone();
        let join_handle = tokio::spawn(async move {
            tokio::select! {
                _ = child.wait() => {
                    state_clone.set_disconnected();
                }
                _ = kill_rx => {
                    let _ = child.start_kill();
                    let _ = child.wait().await;
                }
            }
        });

        let mut handle = ProcessHandle {
            kill_tx: Some(kill_tx),
        };
        handle.shutdown();

        tokio::time::timeout(Duration::from_secs(3), join_handle)
            .await
            .expect("monitor task should finish quickly after shutdown")
            .expect("monitor task panicked");

        // 明示的なshutdownでは state は変更されない(呼び出し側がアプリ終了処理中のため)。
        assert_eq!(state.backend_state(), BackendState::Unlocked);
    }
}

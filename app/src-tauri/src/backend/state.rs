use std::sync::{Arc, Mutex};

use tokio::sync::watch;

/// バックエンド(`bw serve`)とvaultのロック状態。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BackendState {
    /// `bw serve` が起動していない、またはクラッシュ・前提チェック失敗により未接続。
    #[default]
    Disconnected,
    /// `bw serve` は起動しているが、vaultはロックされている。
    Locked,
    /// `bw serve` は起動しており、vaultはアンロック済み。
    Unlocked,
}

#[derive(Debug, Default)]
struct Inner {
    backend: BackendState,
    port: Option<u16>,
    last_error: Option<String>,
    user_email: Option<String>,
}

/// アプリ全体で共有されるロック状態。`tauri::State` として `app.manage()` される。
#[derive(Clone)]
pub struct AppState {
    inner: Arc<Mutex<Inner>>,
    changes: watch::Sender<BackendState>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let (changes, _) = watch::channel(BackendState::Disconnected);
        Self {
            inner: Arc::new(Mutex::new(Inner::default())),
            changes,
        }
    }

    /// バックエンド状態が変化するたびに通知を受け取る購読者を作る
    /// (トレイアイコンの表示更新等に使う)。
    pub fn subscribe(&self) -> watch::Receiver<BackendState> {
        self.changes.subscribe()
    }

    fn set(&self, backend: BackendState) {
        let mut inner = self.inner.lock().expect("AppState mutex poisoned");
        inner.backend = backend;
        drop(inner);
        // 購読者がいなくても送信自体は失敗しない(受信側なしは無視してよい)。
        let _ = self.changes.send(backend);
    }

    pub fn set_disconnected(&self) {
        self.set(BackendState::Disconnected);
    }

    pub fn set_locked(&self) {
        self.set(BackendState::Locked);
    }

    pub fn set_unlocked(&self) {
        self.set(BackendState::Unlocked);
    }

    /// 前提チェック失敗等、致命的なエラーをDisconnectedとして記録する。
    /// `port` も同時にクリアする。エラー状態になった時点でその値が指す
    /// `bw serve` は(起動に失敗した/クラッシュした等により)もはや有効ではないため、
    /// `commands::client_for` が古いポート番号への接続を試みてしまうのを防ぐ。
    pub fn set_error(&self, message: impl Into<String>) {
        let mut inner = self.inner.lock().expect("AppState mutex poisoned");
        inner.backend = BackendState::Disconnected;
        inner.last_error = Some(message.into());
        inner.port = None;
        drop(inner);
        let _ = self.changes.send(BackendState::Disconnected);
    }

    pub fn backend_state(&self) -> BackendState {
        self.inner.lock().expect("AppState mutex poisoned").backend
    }

    pub fn last_error(&self) -> Option<String> {
        self.inner
            .lock()
            .expect("AppState mutex poisoned")
            .last_error
            .clone()
    }

    pub fn set_port(&self, port: u16) {
        self.inner.lock().expect("AppState mutex poisoned").port = Some(port);
    }

    pub fn port(&self) -> Option<u16> {
        self.inner.lock().expect("AppState mutex poisoned").port
    }

    pub fn set_user_email(&self, email: Option<String>) {
        self.inner
            .lock()
            .expect("AppState mutex poisoned")
            .user_email = email;
    }

    pub fn user_email(&self) -> Option<String> {
        self.inner
            .lock()
            .expect("AppState mutex poisoned")
            .user_email
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_disconnected() {
        let state = AppState::new();
        assert_eq!(state.backend_state(), BackendState::Disconnected);
        assert_eq!(state.port(), None);
        assert_eq!(state.user_email(), None);
    }

    #[test]
    fn user_email_setting_and_retrieval() {
        let state = AppState::new();
        assert_eq!(state.user_email(), None);
        state.set_user_email(Some("user@example.com".to_string()));
        assert_eq!(state.user_email().as_deref(), Some("user@example.com"));
        state.set_user_email(None);
        assert_eq!(state.user_email(), None);
    }

    #[test]
    fn unlock_then_lock_transitions() {
        let state = AppState::new();
        state.set_unlocked();
        assert_eq!(state.backend_state(), BackendState::Unlocked);
        state.set_locked();
        assert_eq!(state.backend_state(), BackendState::Locked);
    }

    #[test]
    fn disconnect_after_crash_records_state() {
        let state = AppState::new();
        state.set_unlocked();
        state.set_disconnected();
        assert_eq!(state.backend_state(), BackendState::Disconnected);
    }

    #[test]
    fn error_is_recorded_and_state_becomes_disconnected() {
        let state = AppState::new();
        state.set_error("bw command not found");
        assert_eq!(state.backend_state(), BackendState::Disconnected);
        assert_eq!(state.last_error().as_deref(), Some("bw command not found"));
    }

    #[test]
    fn error_clears_stale_port() {
        let state = AppState::new();
        state.set_port(12345);
        state.set_error("bw serve プロセスが予期せず終了しました。");
        // エラー時点でportは無効になっているため、client_forが古いポートへの
        // 接続を試みないようクリアされていなければならない。
        assert_eq!(state.port(), None);
    }

    #[tokio::test]
    async fn subscribers_are_notified_on_state_change() {
        let state = AppState::new();
        let mut rx = state.subscribe();

        state.set_unlocked();

        rx.changed().await.expect("sender should still be alive");
        assert_eq!(*rx.borrow(), BackendState::Unlocked);
    }
}

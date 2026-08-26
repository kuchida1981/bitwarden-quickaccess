use std::sync::{Arc, Mutex};

/// バックエンド(`bw serve`)とvaultのロック状態。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendState {
    /// `bw serve` が起動していない、またはクラッシュ・前提チェック失敗により未接続。
    Disconnected,
    /// `bw serve` は起動しているが、vaultはロックされている。
    Locked,
    /// `bw serve` は起動しており、vaultはアンロック済み。
    Unlocked,
}

#[derive(Debug, Default)]
struct Inner {
    backend: Option<BackendState>,
    port: Option<u16>,
    last_error: Option<String>,
}

/// アプリ全体で共有されるロック状態。`tauri::State` として `app.manage()` される。
#[derive(Clone)]
pub struct AppState {
    inner: Arc<Mutex<Inner>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                backend: Some(BackendState::Disconnected),
                port: None,
                last_error: None,
            })),
        }
    }

    fn set(&self, backend: BackendState) {
        let mut inner = self.inner.lock().expect("AppState mutex poisoned");
        inner.backend = Some(backend);
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
    pub fn set_error(&self, message: impl Into<String>) {
        let mut inner = self.inner.lock().expect("AppState mutex poisoned");
        inner.backend = Some(BackendState::Disconnected);
        inner.last_error = Some(message.into());
    }

    pub fn backend_state(&self) -> BackendState {
        self.inner
            .lock()
            .expect("AppState mutex poisoned")
            .backend
            .unwrap_or(BackendState::Disconnected)
    }

    pub fn last_error(&self) -> Option<String> {
        self.inner.lock().expect("AppState mutex poisoned").last_error.clone()
    }

    pub fn set_port(&self, port: u16) {
        self.inner.lock().expect("AppState mutex poisoned").port = Some(port);
    }

    pub fn port(&self) -> Option<u16> {
        self.inner.lock().expect("AppState mutex poisoned").port
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
}

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 既定のアイドルタイムアウト(15分)。現行TUIの `BWQA_SESSION_TTL_SECONDS` の
/// デフォルト値を踏襲する。
pub const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(15 * 60);

/// 対象操作(unlock/search/copy/open browser 等)のたびにリセットされる
/// アイドルタイマー。タイムアウトに達したかどうかは `is_expired()` で判定する。
#[derive(Clone)]
pub struct IdleTimer {
    last_activity: Arc<Mutex<Instant>>,
    timeout: Duration,
}

impl IdleTimer {
    pub fn new(timeout: Duration) -> Self {
        Self {
            last_activity: Arc::new(Mutex::new(Instant::now())),
            timeout,
        }
    }

    /// 対象操作が行われたことを記録し、タイマーを起点からやり直す。
    pub fn reset(&self) {
        *self.last_activity.lock().expect("IdleTimer mutex poisoned") = Instant::now();
    }

    /// 最後の操作からタイムアウト時間が経過しているかどうか。
    pub fn is_expired(&self) -> bool {
        self.last_activity
            .lock()
            .expect("IdleTimer mutex poisoned")
            .elapsed()
            >= self.timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_expired_immediately_after_creation() {
        let timer = IdleTimer::new(Duration::from_millis(50));
        assert!(!timer.is_expired());
    }

    #[test]
    fn expires_after_timeout_elapses() {
        let timer = IdleTimer::new(Duration::from_millis(20));
        std::thread::sleep(Duration::from_millis(40));
        assert!(timer.is_expired());
    }

    #[test]
    fn reset_extends_the_deadline() {
        let timer = IdleTimer::new(Duration::from_millis(40));
        std::thread::sleep(Duration::from_millis(20));
        timer.reset();
        std::thread::sleep(Duration::from_millis(20));
        assert!(!timer.is_expired(), "reset直後からの経過時間はまだタイムアウトに達していないはず");
    }
}

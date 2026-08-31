use std::sync::{Arc, Mutex};

/// 直近で本アプリがクリップボードに書き込んだ値を保持する共有state。
/// コピー操作(copy_field)とロック操作(手動/アイドル自動ロック)の双方から
/// 参照し、「クリップボードの中身がアプリが書き込んだ値のままかどうか」を
/// 判定してからクリアするために使う(ユーザーが既に別の値をコピーしている
/// 場合に誤って上書き・消去しないため)。
#[derive(Clone)]
pub struct ClipboardGuard {
    last_written: Arc<Mutex<Option<String>>>,
}

impl Default for ClipboardGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipboardGuard {
    pub fn new() -> Self {
        Self {
            last_written: Arc::new(Mutex::new(None)),
        }
    }

    /// アプリがクリップボードに値を書き込んだ直後に呼ぶ。
    pub fn set(&self, value: String) {
        *self
            .last_written
            .lock()
            .expect("ClipboardGuard mutex poisoned") = Some(value);
    }

    /// クリア処理(遅延クリア・ロック時クリア)が完了した後に呼ぶ。
    /// 保持している機微値をメモリ上から破棄する。
    pub fn clear(&self) {
        *self
            .last_written
            .lock()
            .expect("ClipboardGuard mutex poisoned") = None;
    }

    /// 保持している値が `expected` と一致する場合に限り、内部状態をクリアする。
    /// 一致しない場合(既に別の値で上書きされている場合)は何もしない。
    pub fn clear_if_matches(&self, expected: &str) {
        let mut guard = self
            .last_written
            .lock()
            .expect("ClipboardGuard mutex poisoned");
        if guard.as_deref() == Some(expected) {
            *guard = None;
        }
    }

    /// 現在のクリップボードの中身(current)が、アプリが最後に書き込んだ値と
    /// 一致するかどうかを判定する。一致する場合のみクリアしてよい。
    /// 実際のクリップボードI/Oには一切触れない純粋な判定ロジックにすること
    /// (ユニットテストで検証可能にするため)。
    pub fn should_clear(&self, current: &str) -> bool {
        let guard = self
            .last_written
            .lock()
            .expect("ClipboardGuard mutex poisoned");
        match &*guard {
            Some(last) => last == current,
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_clear_is_false_before_any_set() {
        let guard = ClipboardGuard::new();
        assert!(!guard.should_clear(""));
        assert!(!guard.should_clear("anything"));
    }

    #[test]
    fn should_clear_is_true_when_current_matches_last_written() {
        let guard = ClipboardGuard::new();
        guard.set("password123".to_string());
        assert!(guard.should_clear("password123"));
    }

    #[test]
    fn should_clear_is_false_when_current_differs() {
        let guard = ClipboardGuard::new();
        guard.set("password1".to_string());
        assert!(!guard.should_clear("other-value"));
    }

    #[test]
    fn should_clear_is_false_after_clear() {
        let guard = ClipboardGuard::new();
        guard.set("password123".to_string());
        guard.clear();
        assert!(!guard.should_clear("password123"));
    }

    #[test]
    fn set_overwrites_previous_value() {
        let guard = ClipboardGuard::new();
        guard.set("first-value".to_string());
        guard.set("second-value".to_string());
        assert!(!guard.should_clear("first-value"));
        assert!(guard.should_clear("second-value"));
    }

    #[test]
    fn clear_if_matches_clears_when_value_matches() {
        let guard = ClipboardGuard::new();
        guard.set("a".to_string());
        guard.clear_if_matches("a");
        assert!(!guard.should_clear("a"));
    }

    #[test]
    fn clear_if_matches_does_not_clear_when_value_differs() {
        let guard = ClipboardGuard::new();
        guard.set("a".to_string());
        guard.set("b".to_string());
        guard.clear_if_matches("a");
        assert!(guard.should_clear("b"));
    }
}

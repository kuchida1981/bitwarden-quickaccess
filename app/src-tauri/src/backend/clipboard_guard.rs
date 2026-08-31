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

    /// 現在保持している値のクローンを返す。`clear_clipboard_now` から、直近
    /// 書き込んだ値をexpectedとして `clear_clipboard_if_owned` に渡すために使う。
    pub fn last_value(&self) -> Option<String> {
        self.last_written
            .lock()
            .expect("ClipboardGuard mutex poisoned")
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn last_value_is_none_before_any_set() {
        let guard = ClipboardGuard::new();
        assert_eq!(guard.last_value(), None);
    }

    #[test]
    fn last_value_matches_after_set() {
        let guard = ClipboardGuard::new();
        guard.set("password123".to_string());
        assert_eq!(guard.last_value(), Some("password123".to_string()));
    }

    #[test]
    fn last_value_does_not_match_different_value() {
        let guard = ClipboardGuard::new();
        guard.set("password1".to_string());
        assert_ne!(guard.last_value(), Some("other-value".to_string()));
    }

    #[test]
    fn set_overwrites_previous_value() {
        let guard = ClipboardGuard::new();
        guard.set("first-value".to_string());
        guard.set("second-value".to_string());
        assert_eq!(guard.last_value(), Some("second-value".to_string()));
    }

    #[test]
    fn clear_if_matches_clears_when_value_matches() {
        let guard = ClipboardGuard::new();
        guard.set("a".to_string());
        guard.clear_if_matches("a");
        assert_eq!(guard.last_value(), None);
    }

    #[test]
    fn clear_if_matches_does_not_clear_when_value_differs() {
        let guard = ClipboardGuard::new();
        guard.set("a".to_string());
        guard.clear_if_matches("other");
        assert_eq!(guard.last_value(), Some("a".to_string()));
    }

    #[test]
    fn clear_if_matches_does_not_clear_when_overwritten_by_newer_value() {
        let guard = ClipboardGuard::new();
        guard.set("V1".to_string()); // 1回目のコピー
        guard.set("V2".to_string()); // 30秒以内に2回目のコピー(V1のタイマーがまだ生きている状態を模す)
        guard.clear_if_matches("V1"); // V1のタイマーが発火し、V1をexpectedとしてクリアを試みる
        assert_eq!(guard.last_value(), Some("V2".to_string())); // V2は消えず保持されたまま
    }
}

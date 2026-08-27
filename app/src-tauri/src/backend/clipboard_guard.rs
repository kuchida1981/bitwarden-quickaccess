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

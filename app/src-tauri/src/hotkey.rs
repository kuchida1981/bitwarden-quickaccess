use tauri::AppHandle;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

/// `⇧⌘Space` をグローバルホットキーとして登録する。
/// 登録に失敗しても(他アプリとの衝突等)アプリはクラッシュせず起動を継続し、
/// 失敗内容をログに記録するだけに留める(design.md 決定2)。
pub fn register_quick_access_hotkey(app: &AppHandle) {
    let shortcut = Shortcut::new(Some(Modifiers::SHIFT | Modifiers::SUPER), Code::Space);
    if let Err(err) = app.global_shortcut().register(shortcut) {
        eprintln!(
            "警告: グローバルホットキー(Shift+Cmd+Space)の登録に失敗しました: {err}\n\
             他のアプリと衝突しているか、macOSのシステム設定 > プライバシーとセキュリティ > \
             アクセシビリティ でこのアプリの権限が許可されていない可能性があります。"
        );
    }
}

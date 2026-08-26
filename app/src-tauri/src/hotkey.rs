use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

/// `⇧⌘Space` をグローバルホットキーとして登録する。
/// 登録に失敗しても(他アプリとの衝突等)アプリはクラッシュせず起動を継続し、
/// 失敗内容をログに記録し、エラーを返す(design.md 決定2)。
pub fn register_quick_access_hotkey(app: &AppHandle) -> Result<(), String> {
    let lang = *app.state::<crate::i18n::Lang>().inner();
    let m = crate::i18n::messages(lang);
    let shortcut = Shortcut::new(Some(Modifiers::SHIFT | Modifiers::SUPER), Code::Space);
    if let Err(err) = app.global_shortcut().register(shortcut) {
        let message = m.hotkey_registration_failed.replace("{}", &err.to_string());
        eprintln!("警告: {message}");
        Err(message)
    } else {
        Ok(())
    }
}

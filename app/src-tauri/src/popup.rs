use std::sync::Mutex;

use objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication, NSWorkspace};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

pub const POPUP_LABEL: &str = "popup";

/// ウィンドウが表示されるたびにWebView側へ通知するイベント名。
/// フロントエンドはこれを購読し、ロック状態の再取得・検索ボックスへの
/// 自動フォーカスを行う(`quickaccess-search-ui` design.md 参照)。
pub const POPUP_SHOWN_EVENT: &str = "popup-shown";

/// ポップアップが表示されたまま裏でバックエンド状態(ロック/アンロック等)が
/// 変化した場合に、WebView側へ再判定を促すために発火するイベント名。
/// `POPUP_SHOWN_EVENT` は非表示→表示の遷移時にしか発火しないため、表示中の
/// 状態変化を伝える手段がこれまで存在しなかった(トレイメニューからの明示的
/// ロック等、ポップアップの外からの状態変化で顕在化する)。
pub const BACKEND_STATE_CHANGED_EVENT: &str = "backend-state-changed";

const WIDTH: f64 = 520.0;
const HEIGHT: f64 = 480.0;
const TOP_MARGIN: f64 = 80.0;

/// プレースホルダ内容のポップアップウィンドウを、非表示状態・画面上部中央の位置で作成する。
/// 中身の検索UIは後続change(`quickaccess-search-ui`)で実装する。
pub fn create_popup_window(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    let lang = *app.state::<crate::i18n::Lang>().inner();
    let builder = WebviewWindowBuilder::new(app, POPUP_LABEL, WebviewUrl::App("index.html".into()))
        .title(crate::i18n::messages(lang).app_display_name)
        .inner_size(WIDTH, HEIGHT)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .focused(false);

    let window = builder.build()?;

    let window_for_blur = window.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(false) = event {
            let _ = window_for_blur.hide();
        }
    });

    Ok(window)
}

/// モニターの物理座標系での位置・サイズ・スケールから、ポップアップウィンドウを
/// 表示すべきlogical座標(中央上部)を計算する。`Monitor`型に依存しない
/// 純粋関数にすることで単体テスト可能にする。
fn popup_position_for_monitor(
    monitor_position_physical: (i32, i32),
    monitor_size_physical: (u32, u32),
    scale_factor: f64,
) -> (f64, f64) {
    let position = tauri::PhysicalPosition::new(monitor_position_physical.0, monitor_position_physical.1)
        .to_logical::<f64>(scale_factor);
    let size = tauri::PhysicalSize::new(monitor_size_physical.0, monitor_size_physical.1)
        .to_logical::<f64>(scale_factor);
    let x = position.x + ((size.width - WIDTH) / 2.0).max(0.0);
    let y = position.y + TOP_MARGIN;
    (x, y)
}

/// ホットキー押下時点のカーソル位置が属するディスプレイの中央上部の座標を返す。
/// カーソル位置の取得、またはそこからのディスプレイ特定に失敗した場合は
/// `primary_monitor()` にフォールバックする(design.md 決定1)。
fn compute_popup_position(app: &AppHandle) -> Option<(f64, f64)> {
    let monitor = app
        .cursor_position()
        .ok()
        .and_then(|pos| app.monitor_from_point(pos.x, pos.y).ok().flatten())
        .or_else(|| app.primary_monitor().ok().flatten())?;

    Some(popup_position_for_monitor(
        (monitor.position().x, monitor.position().y),
        (monitor.size().width, monitor.size().height),
        monitor.scale_factor(),
    ))
}

/// ポップアップを表示する直前にフォアグラウンドだった他アプリケーションのPIDを保持する。
/// `NSRunningApplication` インスタンス自体(`Retained<T>`)は `Send + Sync` を満たさず
/// Tauriの `.manage()` では扱えないため、PIDのみを保持し、フォーカス復帰時に
/// `NSRunningApplication::runningApplicationWithProcessIdentifier` で再取得する
/// (design.md 決定3)。
pub struct PreviousFrontmostApp(Mutex<Option<libc::pid_t>>);

impl PreviousFrontmostApp {
    pub fn new() -> Self {
        Self(Mutex::new(None))
    }
}

impl Default for PreviousFrontmostApp {
    fn default() -> Self {
        Self::new()
    }
}

/// ポップアップを表示する直前の最前面アプリケーションのPIDを記録する。
fn record_frontmost_app(app: &AppHandle) {
    let pid = NSWorkspace::sharedWorkspace()
        .frontmostApplication()
        .map(|running_app| running_app.processIdentifier());

    *app
        .state::<PreviousFrontmostApp>()
        .0
        .lock()
        .expect("PreviousFrontmostApp mutex poisoned") = pid;
}

/// ポップアップを閉じた際に、表示直前にフォアグラウンドだったアプリケーションを
/// 再度アクティブ化する(design.md 決定3)。記録が無い、または対象アプリが
/// 既に終了している場合は何もしない。`toggle_popup` の明示的な非表示分岐と
/// `commands::hide_popup` の両方から呼ばれるため冪等に作る(`take()` により
/// 2回目の呼び出しは常に `None` になる)。
pub(crate) fn restore_previous_focus(app: &AppHandle) {
    let pid = app
        .state::<PreviousFrontmostApp>()
        .0
        .lock()
        .expect("PreviousFrontmostApp mutex poisoned")
        .take();

    let Some(pid) = pid else {
        return;
    };

    if let Some(running_app) = NSRunningApplication::runningApplicationWithProcessIdentifier(pid) {
        running_app.activateWithOptions(NSApplicationActivationOptions::empty());
    }
}

/// ポップアップウィンドウの表示/非表示をトグルする。ホットキー押下時に呼ばれる。
pub fn toggle_popup(app: &AppHandle) {
    let Some(window) = app.get_webview_window(POPUP_LABEL) else {
        return;
    };

    let is_visible = window.is_visible().unwrap_or(false);
    if is_visible {
        let _ = window.hide();
        restore_previous_focus(app);
    } else {
        record_frontmost_app(app);
        if let Some((x, y)) = compute_popup_position(app) {
            let _ = window.set_position(tauri::LogicalPosition::new(x, y));
        }
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.emit(POPUP_SHOWN_EVENT, ());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_popup_position_primary_monitor() {
        let pos = popup_position_for_monitor((0, 0), (1920, 1080), 1.0);
        assert_eq!(pos, (700.0, 80.0));
    }

    #[test]
    fn test_popup_position_external_monitor() {
        let pos = popup_position_for_monitor((1920, 0), (1920, 1080), 1.0);
        assert_eq!(pos, (2620.0, 80.0));
    }

    #[test]
    fn test_popup_position_retina_monitor() {
        // 物理位置(0, 0), 物理サイズ(3840, 2160), スケール 2.0 の場合
        // 論理位置(0, 0), 論理サイズ(1920, 1080) になるため、結果は (700.0, 80.0) になるはず
        let pos = popup_position_for_monitor((0, 0), (3840, 2160), 2.0);
        assert_eq!(pos, (700.0, 80.0));
    }
}

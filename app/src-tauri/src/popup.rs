use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

pub const POPUP_LABEL: &str = "popup";

/// ウィンドウが表示されるたびにWebView側へ通知するイベント名。
/// フロントエンドはこれを購読し、ロック状態の再取得・検索ボックスへの
/// 自動フォーカスを行う(`quickaccess-search-ui` design.md 参照)。
pub const POPUP_SHOWN_EVENT: &str = "popup-shown";

const WIDTH: f64 = 420.0;
const HEIGHT: f64 = 480.0;
const TOP_MARGIN: f64 = 80.0;

/// プレースホルダ内容のポップアップウィンドウを、非表示状態・画面上部中央の位置で作成する。
/// 中身の検索UIは後続change(`quickaccess-search-ui`)で実装する。
pub fn create_popup_window(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    let mut builder = WebviewWindowBuilder::new(app, POPUP_LABEL, WebviewUrl::App("index.html".into()))
        .title("bw-quickaccess")
        .inner_size(WIDTH, HEIGHT)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .focused(false);

    if let Ok(Some(monitor)) = app.primary_monitor() {
        let scale = monitor.scale_factor();
        let monitor_size = monitor.size().to_logical::<f64>(scale);
        let x = ((monitor_size.width - WIDTH) / 2.0).max(0.0);
        builder = builder.position(x, TOP_MARGIN);
    }

    let window = builder.build()?;

    let window_for_blur = window.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(false) = event {
            let _ = window_for_blur.hide();
        }
    });

    Ok(window)
}

/// ポップアップウィンドウの表示/非表示をトグルする。ホットキー押下時に呼ばれる。
pub fn toggle_popup(app: &AppHandle) {
    let Some(window) = app.get_webview_window(POPUP_LABEL) else {
        return;
    };

    let is_visible = window.is_visible().unwrap_or(false);
    if is_visible {
        let _ = window.hide();
    } else {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.emit(POPUP_SHOWN_EVENT, ());
    }
}

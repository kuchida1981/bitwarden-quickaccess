use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    image::Image,
    AppHandle, Manager,
};
use tauri_plugin_autostart::ManagerExt;

use bw_quickaccess_gui_lib::backend::state::{AppState, BackendState};

const STATUS_ITEM_ID: &str = "status";
const HOTKEY_STATUS_ITEM_ID: &str = "hotkey_status";
const AUTOSTART_ITEM_ID: &str = "autostart";
const VERSION_ITEM_ID: &str = "version";
const QUIT_ITEM_ID: &str = "quit";

/// `Cargo.toml` の version を単一の情報源とする(`tauri.conf.json` はversionを
/// 明示せず、Tauriがビルド時にCargo.tomlの値を採用する。design.md 決定3)。
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

fn status_label(state: BackendState) -> &'static str {
    match state {
        BackendState::Disconnected => "状態: 未接続",
        BackendState::Locked => "状態: ロック中",
        BackendState::Unlocked => "状態: アンロック済み",
    }
}

fn icon_bytes_for(state: BackendState) -> &'static [u8] {
    match state {
        BackendState::Disconnected => include_bytes!("../icons/tray-disconnected.png"),
        BackendState::Locked => include_bytes!("../icons/tray-locked.png"),
        BackendState::Unlocked => include_bytes!("../icons/tray-unlocked.png"),
    }
}

/// メニューバー常駐アイコンとコンテキストメニュー(ロック状態表示・自動起動トグル・終了)を構築する。
/// バックエンドのロック状態変化を購読し、アイコンとステータス表示を更新し続ける。
pub fn setup_tray(app: &AppHandle, hotkey_warning: Option<&str>) -> tauri::Result<()> {
    let state = app.state::<AppState>().inner().clone();
    let initial = state.backend_state();

    let status_item = MenuItem::with_id(app, STATUS_ITEM_ID, status_label(initial), false, None::<&str>)?;

    let hotkey_text = match hotkey_warning {
        None => "ホットキー: ⇧⌘Space".to_string(),
        Some(reason) => format!("⚠ ホットキー未登録: {reason}"),
    };
    let hotkey_item = MenuItem::with_id(app, HOTKEY_STATUS_ITEM_ID, &hotkey_text, false, None::<&str>)?;

    let autostart_enabled = app.autolaunch().is_enabled().unwrap_or(false);
    let autostart_item = CheckMenuItem::with_id(
        app,
        AUTOSTART_ITEM_ID,
        "ログイン時に自動起動",
        true,
        autostart_enabled,
        None::<&str>,
    )?;

    let quit_item = MenuItem::with_id(app, QUIT_ITEM_ID, "終了", true, None::<&str>)?;

    let version_item = MenuItem::with_id(
        app,
        VERSION_ITEM_ID,
        format!("バージョン: {APP_VERSION}"),
        false,
        None::<&str>,
    )?;

    let menu = Menu::with_items(
        app,
        &[
            &status_item,
            &hotkey_item,
            &PredefinedMenuItem::separator(app)?,
            &autostart_item,
            &PredefinedMenuItem::separator(app)?,
            &version_item,
            &quit_item,
        ],
    )?;

    let autostart_item_for_menu = autostart_item.clone();
    let tray = TrayIconBuilder::new()
        .icon(Image::from_bytes(icon_bytes_for(initial))?)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            QUIT_ITEM_ID => app.exit(0),
            AUTOSTART_ITEM_ID => {
                let autolaunch = app.autolaunch();
                let currently_enabled = autolaunch.is_enabled().unwrap_or(false);
                let result = if currently_enabled {
                    autolaunch.disable()
                } else {
                    autolaunch.enable()
                };
                if let Err(err) = result {
                    eprintln!("自動起動設定の切り替えに失敗しました: {err}");
                }
                // enable()/disable() の戻り値だけを信用せず、実際の状態を再取得してチェック状態に反映する
                let actual_enabled = app.autolaunch().is_enabled().unwrap_or(currently_enabled);
                let _ = autostart_item_for_menu.set_checked(actual_enabled);
            }
            _ => {}
        })
        .build(app)?;

    let mut rx = state.subscribe();
    let status_item_for_task = status_item.clone();
    tauri::async_runtime::spawn(async move {
        while rx.changed().await.is_ok() {
            let new_state = *rx.borrow();
            let _ = status_item_for_task.set_text(status_label(new_state));
            if let Ok(image) = Image::from_bytes(icon_bytes_for(new_state)) {
                let _ = tray.set_icon(Some(image));
            }
        }
    });

    Ok(())
}

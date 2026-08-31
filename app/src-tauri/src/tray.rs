use tauri::{
    image::Image,
    menu::{CheckMenuItem, IsMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager, Wry,
};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_opener::OpenerExt;

use bw_quickaccess_gui_lib::backend::{
    clipboard_guard::ClipboardGuard,
    state::{AppState, BackendState},
};

const STATUS_ITEM_ID: &str = "status";
const HOTKEY_STATUS_ITEM_ID: &str = "hotkey_status";
const OPEN_QUICKACCESS_ITEM_ID: &str = "open_quickaccess";
const AUTOSTART_ITEM_ID: &str = "autostart";
const LOCK_ITEM_ID: &str = "lock";
const ABOUT_ITEM_ID: &str = "about";
const REPO_LINK_ITEM_ID: &str = "repo_link";
const QUIT_ITEM_ID: &str = "quit";

/// `build.rs` で `git describe` から動的に導出されたバージョン文字列を参照する。
/// リリースタグちょうどのビルドでは `v1.1.0`、開発中のセルフビルドでは `v1.1.0-N-gXXXXXXX` 形式になる。
const APP_VERSION: &str = env!("BWQA_DISPLAY_VERSION");

fn status_label(m: &crate::i18n::Messages, state: BackendState) -> &'static str {
    match state {
        BackendState::Disconnected => m.status_disconnected,
        BackendState::Locked => m.status_locked,
        BackendState::Unlocked => m.status_unlocked,
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
    let lang = *app.state::<crate::i18n::Lang>().inner();
    let m = crate::i18n::messages(lang);
    let initial = state.backend_state();

    let status_item = MenuItem::with_id(
        app,
        STATUS_ITEM_ID,
        status_label(m, initial),
        false,
        None::<&str>,
    )?;

    // ホットキーが正常に登録できている場合、その旨は後述の「クイックアクセスを開く」項目の
    // ラベルに併記されるため冗長になる。未登録(失敗)の場合の警告のみメニューに表示する。
    let hotkey_item = hotkey_warning
        .map(|reason| {
            let hotkey_text = m.hotkey_unregistered_prefix.replace("{}", reason);
            MenuItem::with_id(
                app,
                HOTKEY_STATUS_ITEM_ID,
                &hotkey_text,
                false,
                None::<&str>,
            )
        })
        .transpose()?;

    let open_quickaccess_item = MenuItem::with_id(
        app,
        OPEN_QUICKACCESS_ITEM_ID,
        m.open_quickaccess_label,
        true,
        None::<&str>,
    )?;

    let autostart_enabled = app.autolaunch().is_enabled().unwrap_or(false);
    let autostart_item = CheckMenuItem::with_id(
        app,
        AUTOSTART_ITEM_ID,
        m.autostart_label,
        true,
        autostart_enabled,
        None::<&str>,
    )?;

    let lock_item = MenuItem::with_id(
        app,
        LOCK_ITEM_ID,
        m.lock_now_label,
        initial == BackendState::Unlocked,
        None::<&str>,
    )?;

    let quit_item = MenuItem::with_id(app, QUIT_ITEM_ID, m.quit_label, true, None::<&str>)?;

    let about_item = MenuItem::with_id(
        app,
        ABOUT_ITEM_ID,
        format!("{} {}", app.package_info().name, APP_VERSION),
        false,
        None::<&str>,
    )?;

    let repo_link_item = MenuItem::with_id(
        app,
        REPO_LINK_ITEM_ID,
        m.repo_link_label,
        true,
        None::<&str>,
    )?;

    let separator1 = PredefinedMenuItem::separator(app)?;
    let separator2 = PredefinedMenuItem::separator(app)?;
    let mut menu_items: Vec<&dyn IsMenuItem<Wry>> = vec![&status_item];
    if let Some(hotkey_item) = &hotkey_item {
        menu_items.push(hotkey_item);
    }
    menu_items.extend::<[&dyn IsMenuItem<Wry>; 8]>([
        &open_quickaccess_item,
        &separator1,
        &autostart_item,
        &lock_item,
        &separator2,
        &about_item,
        &repo_link_item,
        &quit_item,
    ]);
    let menu = Menu::with_items(app, &menu_items)?;

    let autostart_item_for_menu = autostart_item.clone();
    let tray = TrayIconBuilder::new()
        .icon(Image::from_bytes(icon_bytes_for(initial))?)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            QUIT_ITEM_ID => app.exit(0),
            OPEN_QUICKACCESS_ITEM_ID => crate::popup::toggle_popup(app),
            REPO_LINK_ITEM_ID => {
                if let Err(err) = app.opener().open_url(
                    "https://github.com/kuchida1981/bitwarden-quickaccess",
                    None::<&str>,
                ) {
                    eprintln!("リポジトリページを開けませんでした: {err}");
                }
            }
            LOCK_ITEM_ID => {
                let app_handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(err) = crate::commands::lock(
                        app_handle.clone(),
                        app_handle.state::<AppState>(),
                        app_handle.state::<ClipboardGuard>(),
                    )
                    .await
                    {
                        eprintln!("ロックに失敗しました: {err}");
                    }
                });
            }
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
    let lock_item_for_task = lock_item.clone();
    tauri::async_runtime::spawn(async move {
        while rx.changed().await.is_ok() {
            let new_state = *rx.borrow();
            let _ = status_item_for_task.set_text(status_label(m, new_state));
            let _ = lock_item_for_task.set_enabled(new_state == BackendState::Unlocked);
            if let Ok(image) = Image::from_bytes(icon_bytes_for(new_state)) {
                let _ = tray.set_icon(Some(image));
            }
        }
    });

    Ok(())
}

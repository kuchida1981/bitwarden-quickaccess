use bw_quickaccess_gui_lib::backend::{
    http_client::{BwServeClient, VaultItemSummary},
    idle::IdleTimer,
    state::{AppState, BackendState},
};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_opener::OpenerExt;

use crate::popup;

fn client_for(state: &AppState) -> Result<BwServeClient, String> {
    let port = state
        .port()
        .ok_or_else(|| "バックエンドサービスの準備がまだできていません。".to_string())?;
    Ok(BwServeClient::new(port))
}

/// WebView側がロック中/アンロック済みどちらの画面を表示すべきか判定するために呼ぶ。
#[tauri::command]
pub fn get_lock_state(state: tauri::State<'_, AppState>) -> &'static str {
    match state.backend_state() {
        BackendState::Disconnected => "disconnected",
        BackendState::Locked => "locked",
        BackendState::Unlocked => "unlocked",
    }
}

#[tauri::command]
pub fn get_ui_locale(lang: tauri::State<'_, crate::i18n::Lang>) -> &'static str {
    match *lang {
        crate::i18n::Lang::Ja => "ja",
        crate::i18n::Lang::En => "en",
    }
}

/// アンロックフォームの送信から呼ばれる。成功時はアプリ内部状態も更新する。
#[tauri::command]
pub async fn unlock(
    state: tauri::State<'_, AppState>,
    idle: tauri::State<'_, IdleTimer>,
    password: String,
) -> Result<(), String> {
    idle.reset();
    let client = client_for(&state)?;
    client.unlock(&password).await.map_err(|err| err.to_string())?;
    state.set_unlocked();
    Ok(())
}

/// 検索ボックスの入力(デバウンス後)から呼ばれる。
#[tauri::command]
pub async fn search_items(
    state: tauri::State<'_, AppState>,
    idle: tauri::State<'_, IdleTimer>,
    query: String,
) -> Result<Vec<VaultItemSummary>, String> {
    idle.reset();
    let client = client_for(&state)?;
    client.search_items(&query).await.map_err(|err| err.to_string())
}

/// フォーカス行のユーザー名/パスワード/TOTPをクリップボードにコピーする。
/// 平文の値はWebView側JSには一切渡さず、Rustコア内で取得しそのままクリップボードへ書き込む。
#[tauri::command]
pub async fn copy_field(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    idle: tauri::State<'_, IdleTimer>,
    item_id: String,
    field: String,
) -> Result<(), String> {
    idle.reset();
    let client = client_for(&state)?;

    let value = match field.as_str() {
        "username" => {
            let item = client.get_item(&item_id).await.map_err(|err| err.to_string())?;
            item.login
                .and_then(|login| login.username)
                .ok_or_else(|| "ユーザー名が設定されていません。".to_string())?
        }
        "password" => {
            let item = client.get_item(&item_id).await.map_err(|err| err.to_string())?;
            item.login
                .and_then(|login| login.password)
                .ok_or_else(|| "パスワードが設定されていません。".to_string())?
        }
        "totp" => client.get_totp(&item_id).await.map_err(|err| err.to_string())?,
        other => return Err(format!("不明なフィールドです: {other}")),
    };

    app.clipboard().write_text(value).map_err(|err| err.to_string())
}

/// フォーカス行のURL(login.urisの先頭要素)をデフォルトブラウザで開く。
#[tauri::command]
pub async fn open_in_browser(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    idle: tauri::State<'_, IdleTimer>,
    item_id: String,
) -> Result<(), String> {
    idle.reset();
    let client = client_for(&state)?;
    let item = client.get_item(&item_id).await.map_err(|err| err.to_string())?;
    let url = item
        .login
        .and_then(|login| login.uris.into_iter().next())
        .and_then(|uri| uri.uri)
        .ok_or_else(|| "URLが設定されていません。".to_string())?;

    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|err| err.to_string())
}

/// コピー/ブラウザ起動アクション実行後、フィードバック表示を挟んでポップアップを閉じる際に呼ばれる。
#[tauri::command]
pub fn hide_popup(app: tauri::AppHandle) {
    use tauri::Manager;
    if let Some(window) = app.get_webview_window(popup::POPUP_LABEL) {
        let _ = window.hide();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_for_fails_when_backend_not_ready() {
        let state = AppState::new();
        assert!(client_for(&state).is_err());
    }

    #[test]
    fn client_for_succeeds_once_port_is_set() {
        let state = AppState::new();
        state.set_port(12345);
        assert!(client_for(&state).is_ok());
    }
}

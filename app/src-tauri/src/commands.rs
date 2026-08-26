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

#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchResultItem {
    pub id: String,
    pub name: String,
    pub username: Option<String>,
    pub has_password: bool,
    pub has_totp: bool,
    pub has_url: bool,
}

impl From<VaultItemSummary> for SearchResultItem {
    fn from(item: VaultItemSummary) -> Self {
        let username = item.login.as_ref().and_then(|l| l.username.clone());
        let has_password = item.login.as_ref().and_then(|l| l.password.as_ref()).is_some();
        let has_totp = item.login.as_ref().and_then(|l| l.totp.as_ref()).is_some();
        let has_url = item
            .login
            .as_ref()
            .map(|l| l.uris.iter().any(|u| u.uri.is_some()))
            .unwrap_or(false);

        Self {
            id: item.id,
            name: item.name,
            username,
            has_password,
            has_totp,
            has_url,
        }
    }
}

/// 検索ボックスの入力(デバウンス後)から呼ばれる。
#[tauri::command]
pub async fn search_items(
    state: tauri::State<'_, AppState>,
    idle: tauri::State<'_, IdleTimer>,
    query: String,
) -> Result<Vec<SearchResultItem>, String> {
    idle.reset();
    let client = client_for(&state)?;
    let items = client.search_items(&query).await.map_err(|err| err.to_string())?;
    Ok(items.into_iter().map(SearchResultItem::from).collect())
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
    popup::restore_previous_focus(&app);
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

    #[test]
    fn test_search_result_item_from_vault_item_summary() {
        use bw_quickaccess_gui_lib::backend::http_client::{LoginDetail, UriEntry};

        // 1. すべて持つ
        let item_all = VaultItemSummary {
            id: "1".to_string(),
            name: "Item All".to_string(),
            login: Some(LoginDetail {
                username: Some("user1".to_string()),
                password: Some("pass1".to_string()),
                totp: Some("totp1".to_string()),
                uris: vec![UriEntry {
                    uri: Some("https://example.com".to_string()),
                }],
            }),
        };
        let res_all = SearchResultItem::from(item_all);
        assert_eq!(res_all.id, "1");
        assert_eq!(res_all.name, "Item All");
        assert_eq!(res_all.username, Some("user1".to_string()));
        assert!(res_all.has_password);
        assert!(res_all.has_totp);
        assert!(res_all.has_url);

        // 2. loginがNone
        let item_none = VaultItemSummary {
            id: "2".to_string(),
            name: "Item None".to_string(),
            login: None,
        };
        let res_none = SearchResultItem::from(item_none);
        assert_eq!(res_none.id, "2");
        assert_eq!(res_none.name, "Item None");
        assert_eq!(res_none.username, None);
        assert!(!res_none.has_password);
        assert!(!res_none.has_totp);
        assert!(!res_none.has_url);

        // 3. loginはあるがpassword/totpがNone、urisが空配列
        let item_empty_login = VaultItemSummary {
            id: "3".to_string(),
            name: "Item Empty Login".to_string(),
            login: Some(LoginDetail {
                username: None,
                password: None,
                totp: None,
                uris: vec![],
            }),
        };
        let res_empty_login = SearchResultItem::from(item_empty_login);
        assert_eq!(res_empty_login.id, "3");
        assert_eq!(res_empty_login.name, "Item Empty Login");
        assert_eq!(res_empty_login.username, None);
        assert!(!res_empty_login.has_password);
        assert!(!res_empty_login.has_totp);
        assert!(!res_empty_login.has_url);
    }
}

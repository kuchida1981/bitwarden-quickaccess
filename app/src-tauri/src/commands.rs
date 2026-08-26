use bw_quickaccess_gui_lib::backend::{
    http_client::{BwServeClient, VaultItemSummary},
    state::{AppState, BackendState},
};

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

/// アンロックフォームの送信から呼ばれる。成功時はアプリ内部状態も更新する。
#[tauri::command]
pub async fn unlock(state: tauri::State<'_, AppState>, password: String) -> Result<(), String> {
    let client = client_for(&state)?;
    client.unlock(&password).await.map_err(|err| err.to_string())?;
    state.set_unlocked();
    Ok(())
}

/// 検索ボックスの入力(デバウンス後)から呼ばれる。
#[tauri::command]
pub async fn search_items(
    state: tauri::State<'_, AppState>,
    query: String,
) -> Result<Vec<VaultItemSummary>, String> {
    let client = client_for(&state)?;
    client.search_items(&query).await.map_err(|err| err.to_string())
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

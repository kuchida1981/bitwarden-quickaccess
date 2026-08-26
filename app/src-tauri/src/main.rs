// トレイ常駐・グローバルホットキー・ポップアップウィンドウの骨格。
// 検索UI・コピー操作は後続change(`quickaccess-search-ui` / `credential-actions-autolock`)で追加する。

mod hotkey;
mod popup;
mod tray;

use std::sync::Mutex;
use std::time::Duration;

use bw_quickaccess_gui_lib::backend::{
    http_client::{BwServeClient, LockStatus},
    preflight, process,
    state::AppState,
};
use tauri::Manager;
use tauri_plugin_global_shortcut::ShortcutState;

struct ManagedProcess(Mutex<Option<process::ProcessHandle>>);

fn main() {
    let app_state = AppState::new();

    let app = tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        popup::toggle_popup(app);
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(app_state)
        .manage(ManagedProcess(Mutex::new(None)))
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            popup::create_popup_window(app.handle())?;
            let hotkey_result = hotkey::register_quick_access_hotkey(app.handle());
            tray::setup_tray(app.handle(), hotkey_result.err().as_deref())?;

            let app_handle = app.handle().clone();
            let state = app.state::<AppState>().inner().clone();
            tauri::async_runtime::spawn(async move {
                start_backend(app_handle, state).await;
            });

            let app_handle_for_signal = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                wait_for_shutdown_signal().await;
                app_handle_for_signal.exit(0);
            });

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building the bw-quickaccess-gui application");

    app.run(|app_handle, event| {
        if let tauri::RunEvent::Exit = event {
            let managed = app_handle.state::<ManagedProcess>();
            let mut guard = managed.0.lock().expect("ManagedProcess mutex poisoned");
            if let Some(mut process_handle) = guard.take() {
                process_handle.shutdown();
            }
        }
    });
}

async fn start_backend(app_handle: tauri::AppHandle, state: AppState) {
    if let Err(err) = preflight::check_bw_cli().await {
        state.set_error(err.to_string());
        return;
    }

    let port = match process::pick_free_port() {
        Ok(port) => port,
        Err(err) => {
            state.set_error(format!("空きポートの確保に失敗しました: {err}"));
            return;
        }
    };

    let (process_handle, _monitor) = match process::spawn_supervised(port, state.clone()) {
        Ok(pair) => pair,
        Err(err) => {
            state.set_error(format!("bw serve の起動に失敗しました: {err}"));
            return;
        }
    };

    state.set_port(port);
    app_handle
        .state::<ManagedProcess>()
        .0
        .lock()
        .expect("ManagedProcess mutex poisoned")
        .replace(process_handle);

    let client = BwServeClient::new(port);
    sync_initial_status(&client, &state).await;
}

/// SIGTERM/SIGINT(`kill` やターミナルでのCtrl-Cなど)を待ち受ける。
/// Tauriの`RunEvent::Exit`はトレイの「終了」メニュー等、`AppHandle::exit()`経由の
/// 正規終了パスでしか発火しないため、シグナル受信時も同じパスに合流させることで
/// `bw serve` 子プロセスが確実に終了されるようにする。
async fn wait_for_shutdown_signal() {
    use tokio::signal::unix::{signal, SignalKind};
    let sigterm = signal(SignalKind::terminate());
    let sigint = signal(SignalKind::interrupt());

    match (sigterm, sigint) {
        (Ok(mut sigterm), Ok(mut sigint)) => {
            tokio::select! {
                _ = sigterm.recv() => {}
                _ = sigint.recv() => {}
            }
        }
        (sigterm_res, sigint_res) => {
            eprintln!(
                "警告: シグナルハンドラの登録に失敗しました (SIGTERM: {:?}, SIGINT: {:?})。\n\
                 OSシグナルによる終了では子プロセス (bw serve) が正常に終了しない可能性があります。\n\
                 アプリ内のトレイメニューなどから終了操作を行ってください。",
                sigterm_res.err(),
                sigint_res.err()
            );
            // 早期リターンによりアプリが即座に終了してしまうのを防ぐため、永久に待機する
            std::future::pending::<()>().await;
        }
    }
}

/// `bw serve` 起動直後は数百ms程度の起動待ちが必要なため、短いリトライで
/// 初回の `/status` を取得し、アプリ内部状態を実際のvaultロック状態に同期する。
/// これは継続的なポーリングではなく、起動時の一度きりの同期である(design.md 決定5)。
async fn sync_initial_status(client: &BwServeClient, state: &AppState) {
    for _ in 0..10 {
        match client.status().await {
            Ok(LockStatus::Locked) => {
                state.set_locked();
                return;
            }
            Ok(LockStatus::Unlocked) => {
                state.set_unlocked();
                return;
            }
            Ok(LockStatus::Unauthenticated) => {
                state.set_error("bw にログインしていません。`bw login` を実行してください。");
                return;
            }
            Err(_) => {
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
    }
    state.set_error("bw serve の起動確認がタイムアウトしました。");
}

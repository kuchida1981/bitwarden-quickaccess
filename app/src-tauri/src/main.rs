// トレイ常駐・グローバルホットキー・ポップアップウィンドウ・検索UI・コピー操作を含むアプリ本体のエントリポイント。

mod commands;
mod hotkey;
mod i18n;
mod popup;
mod tray;

use std::sync::Mutex;
use std::time::Duration;

use bw_quickaccess_gui_lib::backend::{
    clipboard_guard::ClipboardGuard,
    http_client::{BwServeClient, LockStatus},
    idle::{IdleTimer, DEFAULT_IDLE_TIMEOUT},
    preflight, process,
    state::{AppState, BackendState},
};
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::ShortcutState;

/// アイドルタイマーの期限切れをチェックする間隔。タイムアウト(既定15分)に比べ
/// 十分短ければよく、厳密な即時性は求めない。
const IDLE_CHECK_INTERVAL: Duration = Duration::from_secs(30);

struct ManagedProcess(Mutex<Option<process::ProcessHandle>>);

const PATH_MARKER: &str = "__BWQA_PATH__";

fn extract_path_from_marker(stdout: &str) -> Option<&str> {
    let path = stdout.split(PATH_MARKER).nth(1)?.trim();
    if path.is_empty() {
        None
    } else {
        Some(path)
    }
}

/// Finderからの起動やログイン項目からの自動起動では、macOSはログインシェルの
/// PATHを引き継がず `/usr/bin:/bin:/usr/sbin:/sbin` 程度の最小PATHしか渡さない。
/// `bw`(Bitwarden CLI)は典型的にHomebrewの `/opt/homebrew/bin` 等にインストール
/// されており、これが原因で `bw` コマンドが見つからず「バックエンドサービスの
/// 準備ができていません」というエラーになる不具合があった(`cargo run` 等
/// ターミナルからの起動ではシェルのPATHを継承するため再現しなかった)。
/// ユーザーのログインシェルを一度だけ起動してPATHを取得し、このプロセスの
/// 環境変数に反映することで解消する。シェルが応答しない場合に備えタイムアウトを設ける。
///
/// PATHの取得は `printenv PATH`(外部コマンド)の出力をそのまま使う。`echo -n
/// {marker}$PATH` のようにシェル内蔵の変数展開に頼ると、fishでは `$PATH` が
/// コロン区切り文字列ではなくリスト変数として扱われ、要素ごとにマーカーが
/// 重複してPATHが1要素目に切り詰められてしまう不具合があった。`printenv` は
/// OSへエクスポートする際の環境変数(常にコロン区切り)をそのまま出力するため、
/// bash/zsh/fishいずれでも同じ形式で取得できる。
fn run_shell_and_capture_stdout(
    mut command: std::process::Command,
    timeout: Duration,
    poll_interval: Duration,
) -> Option<String> {
    use std::io::Read;

    let mut child = command
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;
    let start = std::time::Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return None;
                }
                let mut stdout_str = String::new();
                if let Some(mut stdout) = child.stdout.take() {
                    let _ = stdout.read_to_string(&mut stdout_str);
                }
                return extract_path_from_marker(&stdout_str).map(|s| s.to_string());
            }
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return None;
                }
                std::thread::sleep(poll_interval);
            }
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    }
}

fn fix_path_env() {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
    let mut cmd = std::process::Command::new(shell);
    cmd.args(["-l", "-c", &format!("echo -n {PATH_MARKER}; printenv PATH")]);

    if let Some(path) =
        run_shell_and_capture_stdout(cmd, Duration::from_secs(3), Duration::from_millis(50))
    {
        // SAFETY: 追加スレッドを持たず、子プロセスのkill・reapが完了してから戻るため、
        // この時点で環境変数に触れる他スレッドは存在しない。
        unsafe {
            std::env::set_var("PATH", path);
        }
    }
}

fn main() {
    fix_path_env();

    let app_state = AppState::new();
    let idle_timer = IdleTimer::new(DEFAULT_IDLE_TIMEOUT);
    let clipboard_guard = ClipboardGuard::new();
    let lang = i18n::resolve_lang();
 
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
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .manage(idle_timer)
        .manage(clipboard_guard)
        .manage(lang)
        .manage(popup::PreviousFrontmostApp::new())
        .manage(ManagedProcess(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            commands::get_lock_state,
            commands::get_backend_error,
            commands::get_current_user,
            commands::get_ui_locale,
            commands::lock,
            commands::unlock,
            commands::search_items,
            commands::copy_field,
            commands::open_in_browser,
            commands::hide_popup,
        ])
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

            let state_for_idle = app.state::<AppState>().inner().clone();
            let idle_for_watcher = app.state::<IdleTimer>().inner().clone();
            let guard_for_idle = app.state::<ClipboardGuard>().inner().clone();
            let app_handle_for_idle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                watch_idle_timeout(state_for_idle, idle_for_watcher, app_handle_for_idle, guard_for_idle).await;
            });

            let state_for_popup_notify = app.state::<AppState>().inner().clone();
            let app_handle_for_popup_notify = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let mut rx = state_for_popup_notify.subscribe();
                while rx.changed().await.is_ok() {
                    if let Some(window) = app_handle_for_popup_notify.get_webview_window(popup::POPUP_LABEL) {
                        if window.is_visible().unwrap_or(false) {
                            let _ = window.emit(popup::BACKEND_STATE_CHANGED_EVENT, ());
                        }
                    }
                }
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

/// `bw serve` を起動し、起動確認(readinessポーリング)が完了するまでの間に子プロセスが
/// 終了した場合は、原因を問わず起動失敗とみなしてポートを再取得のうえ再試行する
/// (`process::MAX_STARTUP_ATTEMPTS` 回まで)。ポート確保のTOCTOU(bind→解放→起動の間の
/// 競合)への対処であり、readinessポーリングそのものの待ち時間は増やさない
/// (design.md の「決定3」参照)。
///
/// `build_command`(ポートを渡して起動コマンドを組み立てる)と `readiness_check`
/// (起動確認が完了するまで待つ)を引数として切り出すことで、実プロセスなしに
/// リトライの分岐(早期終了→リトライ / 起動確認完了→成功)をテストできるようにしている。
///
/// `register_process` は各試行でプロセスをspawnした直後、起動確認を待つ前に呼ばれる。
/// リトライループ全体が終わるまで登録を遅らせると、その間にアプリが終了した場合に
/// 起動済みの子プロセスが `ManagedProcess` に登録されておらずkillされない(孤児化する)
/// ため、試行のたびに最新のハンドルへ差し替える(古いハンドルは既に終了しているので
/// 破棄して問題ない)。
async fn acquire_backend_process<F, R, Fut, Reg>(
    state: &AppState,
    mut build_command: F,
    mut readiness_check: R,
    mut register_process: Reg,
) -> Option<u16>
where
    F: FnMut(u16) -> tokio::process::Command,
    R: FnMut(u16) -> Fut,
    Fut: std::future::Future<Output = ()>,
    Reg: FnMut(process::ProcessHandle),
{
    for attempt in 1..=process::MAX_STARTUP_ATTEMPTS {
        let port = match process::pick_free_port() {
            Ok(port) => port,
            Err(err) => {
                state.set_error(format!("空きポートの確保に失敗しました: {err}"));
                return None;
            }
        };

        let process::StartupHandles {
            process_handle,
            monitor: _monitor,
            exited,
            confirm,
        } = match process::spawn_supervised_for_startup_with_command(build_command(port), state.clone()) {
            Ok(handles) => handles,
            Err(err) => {
                state.set_error(format!("bw serve の起動に失敗しました: {err}"));
                return None;
            }
        };

        register_process(process_handle);
        // readiness_check(実運用では sync_initial_status)は完了時に内部で
        // state.set_locked()/set_unlocked() を呼ぶため、port はそれより前に
        // 記録しておく必要がある(そうしないと「ロック状態はセット済みだが
        // port は未セット」という一瞬の不整合が生まれ、client_for が
        // 「バックエンドサービスの準備がまだできていません」を誤って返しうる)。
        state.set_port(port);

        let mut exited = exited;
        tokio::select! {
            _ = readiness_check(port) => {
                // 成功/タイムアウトいずれの場合も readiness ポーリングは完了しているため、
                // 以後の監視は現行通り「予期せぬ終了→state.set_error()」に切り替える。
                let _ = confirm.send(());
                return Some(port);
            }
            _ = &mut exited => {
                eprintln!(
                    "bw serve がport {port}での起動確認中に終了しました(試行 {attempt}/{})。ポートを再取得してリトライします。",
                    process::MAX_STARTUP_ATTEMPTS
                );
            }
        }
    }

    state.set_error(format!(
        "bw serve の起動に{}回失敗しました。アプリを再起動してください。",
        process::MAX_STARTUP_ATTEMPTS
    ));
    None
}

async fn start_backend(app_handle: tauri::AppHandle, state: AppState) {
    if let Err(err) = preflight::check_bw_cli().await {
        state.set_error(err.to_string());
        return;
    }

    let state_for_readiness = state.clone();
    acquire_backend_process(
        &state,
        process::build_bw_serve_command,
        move |port| {
            let client = BwServeClient::new(port);
            let state = state_for_readiness.clone();
            async move { sync_initial_status(&client, &state).await }
        },
        move |process_handle| {
            app_handle
                .state::<ManagedProcess>()
                .0
                .lock()
                .expect("ManagedProcess mutex poisoned")
                .replace(process_handle);
        },
    )
    .await;
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

/// アイドルタイマーを定期的にチェックし、タイムアウトに達していれば `/lock` を呼んで
/// アンロック済み状態を解除する。タイマーのリセットは各Tauri command側(unlock/
/// search_items/copy_field/open_in_browser)が担い、ここでは期限切れの検知と
/// ロック実行のみを一元的に行う(design.md 決定4)。
async fn watch_idle_timeout(
    state: AppState,
    idle: IdleTimer,
    app_handle: tauri::AppHandle,
    guard: ClipboardGuard,
) {
    loop {
        tokio::time::sleep(IDLE_CHECK_INTERVAL).await;

        if state.backend_state() != BackendState::Unlocked || !idle.is_expired() {
            continue;
        }

        let Some(port) = state.port() else {
            continue;
        };

        let client = BwServeClient::new(port);
        if client.lock().await.is_ok() {
            state.set_locked();
            commands::clear_clipboard_if_owned(&app_handle, &guard);
        }
    }
}

/// `bw serve` 起動直後は数百ms程度の起動待ちが必要なため、短いリトライで
/// 初回の `/status` を取得し、アプリ内部状態を実際のvaultロック状態に同期する。
/// これは継続的なポーリングではなく、起動時の一度きりの同期である(design.md 決定5)。
async fn sync_initial_status(client: &BwServeClient, state: &AppState) {
    for _ in 0..10 {
        match client.status().await {
            Ok(status_info) => match status_info.lock_status {
                LockStatus::Locked => {
                    state.set_locked();
                    state.set_user_email(status_info.user_email);
                    return;
                }
                LockStatus::Unlocked => {
                    state.set_unlocked();
                    state.set_user_email(status_info.user_email);
                    return;
                }
                LockStatus::Unauthenticated => {
                    state.set_error("bw にログインしていません。`bw login` を実行してください。");
                    return;
                }
            },
            Err(_) => {
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
    }
    state.set_error("bw serve の起動確認がタイムアウトしました。");
}

#[cfg(test)]
mod fix_path_env_tests {
    use super::*;

    #[test]
    fn extracts_path_after_marker() {
        let stdout = format!("{PATH_MARKER}/usr/bin:/opt/homebrew/bin");
        assert_eq!(extract_path_from_marker(&stdout), Some("/usr/bin:/opt/homebrew/bin"));
    }

    #[test]
    fn returns_none_when_marker_missing() {
        assert_eq!(extract_path_from_marker("/usr/bin:/bin"), None);
    }

    #[test]
    fn returns_none_when_path_after_marker_is_empty() {
        let stdout = format!("{PATH_MARKER}   \n");
        assert_eq!(extract_path_from_marker(&stdout), None);
    }

    #[test]
    fn ignores_login_shell_noise_before_marker() {
        let stdout = format!("Last login: Mon Jan 1\n{PATH_MARKER}/usr/bin:/opt/homebrew/bin\n");
        assert_eq!(extract_path_from_marker(&stdout), Some("/usr/bin:/opt/homebrew/bin"));
    }

    #[test]
    fn run_shell_success_extracts_path() {
        let mut cmd = std::process::Command::new("sh");
        cmd.args(["-c", &format!("echo -n {PATH_MARKER}; printenv PATH")]);

        let result = run_shell_and_capture_stdout(
            cmd,
            Duration::from_secs(1),
            Duration::from_millis(5),
        );
        assert!(result.is_some());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn run_shell_timeout_kills_and_returns_none_quickly() {
        let mut cmd = std::process::Command::new("sh");
        cmd.args(["-c", "sleep 5"]);

        let start = std::time::Instant::now();
        let result = run_shell_and_capture_stdout(
            cmd,
            Duration::from_millis(50),
            Duration::from_millis(5),
        );
        let elapsed = start.elapsed();

        assert_eq!(result, None);
        assert!(
            elapsed < Duration::from_secs(1),
            "Expected timeout and kill to finish in less than 1s, but took {elapsed:?}"
        );
    }

    #[test]
    fn run_shell_spawn_failure_returns_none() {
        let cmd = std::process::Command::new("/nonexistent-shell-xyz-12345");
        let result = run_shell_and_capture_stdout(
            cmd,
            Duration::from_millis(50),
            Duration::from_millis(5),
        );
        assert_eq!(result, None);
    }
}

#[cfg(test)]
mod acquire_backend_process_tests {
    use super::*;
    use std::process::Stdio;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use tokio::process::Command;

    /// readiness_check が反応するまでの猶予。`sh -c "exit 1"` の実際の終了(通常数十ms)は
    /// 十分前に観測できる長さにしてある。
    const READINESS_DELAY: Duration = Duration::from_millis(300);

    fn exits_immediately(_port: u16) -> Command {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", "exit 1"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        cmd
    }

    fn stays_alive(_port: u16) -> Command {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", "sleep 5"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        cmd
    }

    /// 実際の `bw serve` の代わりに、一定時間後に「起動確認完了」を返すだけの
    /// readiness_check。早期終了する試行では、この時間が来る前に `exited` が
    /// 先に発火するため、実プロセスなしにリトライ分岐を検証できる。
    async fn readiness_after_delay(_port: u16) {
        tokio::time::sleep(READINESS_DELAY).await;
    }

    /// `ProcessHandle` の drop は(`kill_tx` が閉じることで)監視タスクへの
    /// 暗黙のkill指示として働くため、`register_process` で受け取ったハンドルを
    /// テスト関数が終わるまで生かしておく必要がある(即座にdropすると、
    /// まだ生きているはずの子プロセスがそこでkillされてしまう)。
    fn retaining_register(
        store: Arc<Mutex<Vec<process::ProcessHandle>>>,
    ) -> impl FnMut(process::ProcessHandle) {
        move |handle| store.lock().expect("handle store poisoned").push(handle)
    }

    #[tokio::test]
    async fn retries_after_early_exit_then_succeeds() {
        let state = AppState::new();
        let attempts = Arc::new(AtomicUsize::new(0));
        let attempts_for_build = attempts.clone();
        let handles_store = Arc::new(Mutex::new(Vec::new()));

        let result = acquire_backend_process(
            &state,
            move |port| {
                let n = attempts_for_build.fetch_add(1, Ordering::SeqCst);
                if n == 0 {
                    exits_immediately(port)
                } else {
                    stays_alive(port)
                }
            },
            readiness_after_delay,
            retaining_register(handles_store.clone()),
        )
        .await;

        assert!(result.is_some(), "2回目の試行で起動確認に成功するはず");
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
        assert!(state.last_error().is_none());
        // 失敗した1回目も含め、試行のたびにプロセスハンドルが登録される
        // (アプリ終了時に確実にkillできるようにするため)。
        assert_eq!(handles_store.lock().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn gives_up_after_max_attempts() {
        let state = AppState::new();
        let attempts = Arc::new(AtomicUsize::new(0));
        let attempts_for_build = attempts.clone();
        let handles_store = Arc::new(Mutex::new(Vec::new()));

        let result = acquire_backend_process(
            &state,
            move |port| {
                attempts_for_build.fetch_add(1, Ordering::SeqCst);
                exits_immediately(port)
            },
            readiness_after_delay,
            retaining_register(handles_store.clone()),
        )
        .await;

        assert!(result.is_none());
        assert_eq!(
            attempts.load(Ordering::SeqCst),
            process::MAX_STARTUP_ATTEMPTS as usize
        );
        assert!(state.last_error().is_some());
        // 全試行が失敗した場合、最後の(死んだ)試行のポート番号が残ってはならない
        // (client_forが無効なポートへの接続を試みてしまうため)。
        assert_eq!(state.port(), None);
        assert_eq!(
            handles_store.lock().unwrap().len(),
            process::MAX_STARTUP_ATTEMPTS as usize
        );
    }

    #[tokio::test]
    async fn succeeds_immediately_without_retry_when_first_attempt_is_healthy() {
        let state = AppState::new();
        let attempts = Arc::new(AtomicUsize::new(0));
        let attempts_for_build = attempts.clone();

        let handles_store = Arc::new(Mutex::new(Vec::new()));

        let result = acquire_backend_process(
            &state,
            move |port| {
                attempts_for_build.fetch_add(1, Ordering::SeqCst);
                stays_alive(port)
            },
            readiness_after_delay,
            retaining_register(handles_store),
        )
        .await;

        assert!(result.is_some());
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn port_is_recorded_before_readiness_check_runs() {
        // readiness_check(実運用ではsync_initial_status)は内部でstate.set_locked()/
        // set_unlocked()を呼ぶため、readiness_checkが呼ばれる時点で既にstate.port()が
        // セットされていなければならない(でないと「ロック状態はセット済みだがportは
        // 未セット」という不整合ウィンドウが生まれる)。
        let state = AppState::new();
        let handles_store = Arc::new(Mutex::new(Vec::new()));
        let state_for_check = state.clone();

        let result = acquire_backend_process(
            &state,
            stays_alive,
            move |port| {
                assert_eq!(state_for_check.port(), Some(port));
                async {}
            },
            retaining_register(handles_store),
        )
        .await;

        assert!(result.is_some());
    }
}

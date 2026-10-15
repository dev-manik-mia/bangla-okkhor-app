//! BanglaOkkhor desktop app: tray icon, windows and the system-wide typing hook.

mod commands;
#[cfg(target_os = "macos")]
mod macos;
mod settings;
mod state;
mod tray;
mod typing;
mod windows;

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};

use crate::state::AppState;
use crate::windows::Win;

static RETRYING: AtomicBool = AtomicBool::new(false);

/// Install the keyboard hook, retrying in the background until the OS
/// permission it needs has been granted.
pub(crate) fn start_hook(app: &AppHandle) {
    if okkhor_input::is_running() {
        return;
    }
    let handler_app = app.clone();
    match okkhor_input::start(Box::new(move |ev| typing::handle(&handler_app, ev))) {
        Ok(()) => {
            log::info!("keyboard hook running");
            let _ = app.emit("permission", commands::permission_info());
        }
        Err(e) => {
            log::warn!("keyboard hook not started: {e}");
            if RETRYING.swap(true, Ordering::SeqCst) {
                return;
            }
            let app = app.clone();
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(Duration::from_secs(2));
                    if okkhor_input::is_running() {
                        break;
                    }
                    if okkhor_input::permission() == okkhor_input::Permission::Granted {
                        let handler_app = app.clone();
                        if okkhor_input::start(Box::new(move |ev| typing::handle(&handler_app, ev))).is_ok() {
                            let _ = app.emit("permission", commands::permission_info());
                            break;
                        }
                    }
                }
                RETRYING.store(false, Ordering::SeqCst);
            });
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            windows::show(app, Win::Main);
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, None))
        .invoke_handler(tauri::generate_handler![
            commands::bootstrap,
            commands::set_settings,
            commands::save_settings,
            commands::set_mode,
            commands::toggle_mode,
            commands::virtual_key,
            commands::accept_suggestion,
            commands::convert,
            commands::set_encoding,
            commands::open_window,
            commands::hide_window,
            commands::set_recording,
            commands::set_keyboard_focus,
            commands::permission_status,
            commands::request_permission,
            commands::finish_onboarding,
            commands::convert_selection,
            commands::quit,
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let config_dir = app.path().app_config_dir()?;
            app.manage(AppState::new(config_dir));
            let handle = app.handle().clone();

            tray::create(&handle)?;
            // Preferences window: created hidden, it also plays key clicks.
            windows::get_or_build(&handle, Win::Main);

            let settings = handle.state::<AppState>().settings();
            if settings.indicator {
                windows::show(&handle, Win::Indicator);
            }
            if !settings.onboarded {
                windows::show(&handle, Win::Onboarding);
            }
            start_hook(&handle);
            state::watch_frontmost(handle.clone());
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building BanglaOkkhor")
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested { api, code, .. } = event {
                // Closing the last window keeps the tray app running.
                if code.is_none() {
                    api.prevent_exit();
                } else {
                    app.state::<AppState>().save_words();
                }
            }
        });
}

//! BanglaOkkhor desktop app: tray icon, windows and the system-wide typing hook.

mod commands;
mod fonts;
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

/// Passed when the OS starts us at login: stay quietly in the menu bar / tray.
const AUTOSTART_ARG: &str = "--autostart";

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
            tray::update(app);
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
                            log::info!("keyboard hook running");
                            let _ = app.emit("permission", commands::permission_info());
                            tray::update(&app);
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
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec![AUTOSTART_ARG])))
        .invoke_handler(tauri::generate_handler![
            commands::bootstrap,
            commands::set_settings,
            commands::save_settings,
            commands::set_mode,
            commands::toggle_mode,
            commands::set_input,
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
            commands::fix_access_cmd,
            commands::finish_onboarding,
            commands::convert_selection,
            commands::quit,
            fonts::sutonny_font,
            fonts::sutonny_status,
            fonts::import_sutonny_font,
            fonts::remove_sutonny_font,
        ])
        .setup(|app| {

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
            if settings.widget {
                windows::show(&handle, Win::Widget);
            }
            let at_login = std::env::args().any(|a| a == AUTOSTART_ARG);
            if !settings.onboarded {
                windows::show(&handle, Win::Onboarding);
            } else if !at_login {
                // Opened by the user (Dock, Launchpad, Start menu…): show something.
                windows::show(&handle, Win::Main);
            }
            // Started at login: no Dock icon, only the menu bar one.
            windows::sync_dock(&handle, None);
            start_hook(&handle);
            state::watch_frontmost(handle.clone());
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building BanglaOkkhor")
        .run(|app, event| match event {
            // Clicking the Dock icon while running.
            #[cfg(target_os = "macos")]
            tauri::RunEvent::Reopen { .. } => windows::show(app, Win::Main),
            tauri::RunEvent::Exit => app.state::<AppState>().save_words(),
            _ => {}
        });
}

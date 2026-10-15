//! Commands the frontend calls with `invoke`.

use std::sync::atomic::Ordering;
use std::time::Duration;

use okkhor_core::{layout::LAYOUT, to_ansi, to_unicode, Encoding, Key};
use okkhor_input::{Permission, Special};
use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_autostart::ManagerExt;

use crate::settings::Settings;
use crate::state::{AppState, Target};
use crate::windows::{self, Win};
use crate::{tray, typing};

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PermissionInfo {
    pub granted: bool,
    pub message: Option<String>,
    pub running: bool,
}

pub fn permission_info() -> PermissionInfo {
    let (granted, message) = match okkhor_input::permission() {
        Permission::Granted => (true, None),
        Permission::Missing(m) => (false, Some(m)),
    };
    PermissionInfo { granted, message, running: okkhor_input::is_running() }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutKey {
    code: &'static str,
    bn: &'static str,
    bn_shift: &'static str,
    lat: String,
    lat_shift: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bootstrap {
    settings: Settings,
    bangla: bool,
    target: Target,
    suggestions: Vec<String>,
    permission: PermissionInfo,
    platform: &'static str,
    layout: Vec<LayoutKey>,
    version: String,
}

fn platform() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "linux"
    }
}

#[tauri::command]
pub fn bootstrap(app: AppHandle, st: State<AppState>) -> Bootstrap {
    Bootstrap {
        settings: st.settings(),
        bangla: st.is_bangla(),
        target: st.target.lock().unwrap().clone(),
        suggestions: st.suggestions.lock().unwrap().clone(),
        permission: permission_info(),
        platform: platform(),
        layout: LAYOUT
            .iter()
            .map(|(k, n, s)| LayoutKey {
                code: k.code(),
                bn: n,
                bn_shift: s,
                lat: k.latin(false).to_string(),
                lat_shift: k.latin(true).to_string(),
            })
            .collect(),
        version: app.package_info().version.to_string(),
    }
}

fn merge(base: &mut Value, patch: Value) {
    match (base, patch) {
        (Value::Object(b), Value::Object(p)) => {
            for (k, v) in p {
                merge(b.entry(k).or_insert(Value::Null), v);
            }
        }
        (b, p) => *b = p,
    }
}

/// Apply a partial settings object, e.g. `{ "sound": true }`.
#[tauri::command]
pub fn set_settings(app: AppHandle, st: State<AppState>, patch: Value) -> Result<Settings, String> {
    let old = st.settings();
    let mut value = serde_json::to_value(&old).map_err(|e| e.to_string())?;
    merge(&mut value, patch);
    let new: Settings = serde_json::from_value(value).map_err(|e| e.to_string())?;
    *st.settings.write().unwrap() = new.clone();
    st.save_settings();
    apply_settings(&app, &st, &old, &new);
    Ok(new)
}

pub fn apply_settings(app: &AppHandle, st: &AppState, old: &Settings, new: &Settings) {
    {
        let mut engine = st.engine.lock().unwrap();
        engine.options = new.engine_options();
        engine.layout = new.layout;
    }
    if old.layout != new.layout {
        st.end_run(app);
        tray::update(app);
    }
    if old.launch != new.launch {
        let al = app.autolaunch();
        let r = if new.launch { al.enable() } else { al.disable() };
        if let Err(e) = r {
            log::warn!("autostart: {e}");
        }
    }
    if old.widget != new.widget {
        if new.widget {
            windows::show(app, Win::Widget);
        } else {
            windows::hide(app, Win::Widget);
        }
    }
    if old.indicator != new.indicator || old.widget != new.widget {
        tray::update(app);
    }
    if old.indicator != new.indicator {
        if new.indicator {
            windows::show(app, Win::Indicator);
        } else {
            windows::hide(app, Win::Indicator);
        }
    }
    if !new.sugg {
        st.set_suggestions(app, Vec::new());
    }
    if old.lang != new.lang || old.shortcuts != new.shortcuts {
        tray::update(app);
    }
    let _ = app.emit("settings", new.clone());
}

/// Write the current settings to disk (e.g. after the widget was moved).
#[tauri::command]
pub fn save_settings(st: State<AppState>) {
    st.save_settings();
}

#[tauri::command]
pub fn set_mode(app: AppHandle, st: State<AppState>, bangla: bool) {
    st.set_mode(&app, bangla);
}

/// "bijoy", "avro" or "en".
#[tauri::command]
pub fn set_input(app: AppHandle, st: State<AppState>, input: String) {
    st.set_input(&app, &input);
}

#[tauri::command]
pub fn toggle_mode(app: AppHandle, st: State<AppState>) -> bool {
    let next = !st.is_bangla();
    st.set_mode(&app, next);
    next
}

#[tauri::command]
pub fn virtual_key(app: AppHandle, st: State<AppState>, code: String, shift: bool) {
    match code.as_str() {
        "Backspace" => typing::virtual_special(&app, &st, Special::Backspace),
        "Enter" => typing::virtual_special(&app, &st, Special::Enter),
        "Tab" => typing::virtual_special(&app, &st, Special::Tab),
        _ => {
            if let Some(key) = Key::from_code(&code) {
                typing::virtual_key(&app, &st, key, shift);
            }
        }
    }
}

#[tauri::command]
pub fn accept_suggestion(app: AppHandle, st: State<AppState>, word: String) {
    typing::accept(&app, &st, &word);
}

#[tauri::command]
pub fn convert(text: String, to: String) -> String {
    if to == "ansi" {
        to_ansi(&text)
    } else {
        to_unicode(&text)
    }
}

#[tauri::command]
pub fn set_encoding(app: AppHandle, st: State<AppState>, encoding: Encoding) {
    st.set_encoding(&app, encoding);
}

#[tauri::command]
pub fn open_window(app: AppHandle, name: String) {
    if let Some(w) = Win::from_label(&name) {
        windows::show(&app, w);
    }
}

#[tauri::command]
pub fn hide_window(app: AppHandle, name: String) {
    if let Some(w) = Win::from_label(&name) {
        windows::hide(&app, w);
    }
}

#[tauri::command]
pub fn set_recording(st: State<AppState>, on: bool) {
    st.recording.store(on, Ordering::SeqCst);
}

#[tauri::command]
pub fn set_keyboard_focus(app: AppHandle, on: bool) {
    windows::set_keyboard_focus(&app, on);
}

#[tauri::command]
pub fn permission_status() -> PermissionInfo {
    permission_info()
}

/// Show or hide the mode pill / floating widget and remember the choice.
/// Typing keeps working either way.
pub fn set_floating(app: &AppHandle, which: Win, on: bool) {
    let st = app.state::<AppState>();
    let old = st.settings();
    let mut new = old.clone();
    match which {
        Win::Widget => new.widget = on,
        Win::Indicator => new.indicator = on,
        _ => return,
    }
    *st.settings.write().unwrap() = new.clone();
    st.save_settings();
    apply_settings(app, &st, &old, &new);
}

/// Typing is off: ask the OS for keyboard access and open the right settings page.
pub fn fix_access(app: &AppHandle) {
    okkhor_input::request_permission();
    #[cfg(target_os = "macos")]
    {
        use tauri_plugin_opener::OpenerExt;
        let _ = app.opener().open_url("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility", None::<&str>);
    }
    #[cfg(not(target_os = "macos"))]
    windows::show(app, Win::Onboarding);
    crate::start_hook(app);
}

#[tauri::command]
pub fn fix_access_cmd(app: AppHandle) {
    fix_access(&app);
}

#[tauri::command]
pub fn request_permission(app: AppHandle) -> PermissionInfo {
    okkhor_input::request_permission();
    crate::start_hook(&app);
    permission_info()
}

#[tauri::command]
pub fn finish_onboarding(app: AppHandle, st: State<AppState>) {
    let old = st.settings();
    let mut new = old.clone();
    new.onboarded = true;
    *st.settings.write().unwrap() = new.clone();
    st.save_settings();
    apply_settings(&app, &st, &old, &new);
    windows::hide(&app, Win::Onboarding);
    if let Some(w) = app.get_webview_window(Win::Onboarding.label()) {
        let _ = w.close();
    }
    windows::show(&app, Win::Keyboard);
}

#[tauri::command]
pub fn quit(app: AppHandle, st: State<AppState>) {
    st.save_words();
    app.exit(0);
}

fn has_bangla(s: &str) -> bool {
    s.chars().any(|c| ('\u{0980}'..='\u{09FF}').contains(&c))
}

/// Convert the selection in the focused app (Unicode ⇄ ANSI) through the
/// clipboard. Returns false when nothing was selected.
#[tauri::command]
pub async fn convert_selection(app: AppHandle) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || {
        const SENTINEL: &str = "\u{2063}okkhor-empty\u{2063}";
        let mut cb = arboard::Clipboard::new().map_err(|e| e.to_string())?;
        let saved = cb.get_text().ok();
        cb.set_text(SENTINEL).map_err(|e| e.to_string())?;
        okkhor_input::send_copy();
        std::thread::sleep(Duration::from_millis(250));
        let selected = cb.get_text().unwrap_or_default();
        if selected == SENTINEL || selected.is_empty() {
            if let Some(s) = saved {
                let _ = cb.set_text(s);
            }
            return Ok(false);
        }
        let converted = if has_bangla(&selected) { to_ansi(&selected) } else { to_unicode(&selected) };
        cb.set_text(converted).map_err(|e| e.to_string())?;
        okkhor_input::send_paste();
        std::thread::sleep(Duration::from_millis(400));
        if let Some(s) = saved {
            let _ = cb.set_text(s);
        }
        let _ = app.emit("converted", ());
        Ok(true)
    })
    .await
    .map_err(|e| e.to_string())?
}

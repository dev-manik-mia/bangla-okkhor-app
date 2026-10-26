//! Creating and showing the app's windows.

use std::sync::atomic::Ordering;

use tauri::utils::config::WindowEffectsConfig;
use tauri::window::{Effect, EffectState};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent};

use crate::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Win {
    /// Preferences. Created at startup and hidden on close.
    Main,
    Keyboard,
    Converter,
    Onboarding,
    Widget,
    Indicator,
}

impl Win {
    pub fn label(self) -> &'static str {
        match self {
            Win::Main => "main",
            Win::Keyboard => "keyboard",
            Win::Converter => "converter",
            Win::Onboarding => "onboarding",
            Win::Widget => "widget",
            Win::Indicator => "indicator",
        }
    }

    pub fn from_label(s: &str) -> Option<Win> {
        [Win::Main, Win::Keyboard, Win::Converter, Win::Onboarding, Win::Widget, Win::Indicator]
            .into_iter()
            .find(|w| w.label() == s)
    }

    /// Floating tool windows never take focus from the app being typed in.
    fn floating(self) -> bool {
        matches!(self, Win::Keyboard | Win::Widget | Win::Indicator)
    }

    fn chromeless(self) -> bool {
        matches!(self, Win::Widget | Win::Indicator)
    }

    fn size(self) -> (f64, f64) {
        match self {
            Win::Main => (720.0, 500.0),
            Win::Keyboard => (880.0, 520.0),
            Win::Converter => (780.0, 380.0),
            Win::Onboarding => (560.0, 480.0),
            Win::Widget => (300.0, 156.0),
            Win::Indicator => (292.0, 48.0),
        }
    }

    fn title(self) -> &'static str {
        match self {
            Win::Main => "BanglaOkkhor Preferences",
            Win::Keyboard => "BanglaOkkhor Keyboard",
            Win::Converter => "Converter",
            Win::Onboarding => "Welcome to BanglaOkkhor",
            Win::Widget => "BanglaOkkhor Widget",
            Win::Indicator => "BanglaOkkhor Mode",
        }
    }
}

fn effects(win: Win) -> Option<WindowEffectsConfig> {
    if cfg!(target_os = "macos") {
        let (effect, radius) = if win.chromeless() { (Effect::Popover, Some(if win == Win::Indicator { 24.0 } else { 16.0 })) } else { (Effect::UnderWindowBackground, None) };
        Some(WindowEffectsConfig { effects: vec![effect], state: Some(EffectState::Active), radius, color: None, ..Default::default() })
    } else if cfg!(target_os = "windows") {
        let effect = if win.chromeless() { Effect::Acrylic } else { Effect::Mica };
        Some(WindowEffectsConfig { effects: vec![effect], state: None, radius: None, color: None, ..Default::default() })
    } else {
        None
    }
}

fn build(app: &AppHandle, win: Win) -> tauri::Result<WebviewWindow> {
    let (w, h) = win.size();
    let url = WebviewUrl::App(format!("index.html#{}", win.label()).into());
    let mut b = WebviewWindowBuilder::new(app, win.label(), url)
        .title(win.title())
        .inner_size(w, h)
        .resizable(matches!(win, Win::Converter | Win::Keyboard))
        .visible(false);
    if let Some(fx) = effects(win) {
        b = b.transparent(true).effects(fx);
    } else if win.chromeless() {
        b = b.transparent(true);
    }
    if win.chromeless() {
        b = b.decorations(false).shadow(false).skip_taskbar(true).always_on_top(true).visible_on_all_workspaces(true);
    } else {
        b = b.center();
        #[cfg(target_os = "macos")]
        {
            b = b.title_bar_style(tauri::TitleBarStyle::Overlay).hidden_title(true);
        }
    }
    if win == Win::Keyboard {
        b = b.always_on_top(true).visible_on_all_workspaces(true).min_inner_size(640.0, 420.0);
    }
    if win.floating() {
        // accept_first_mouse: the first click on an inactive panel acts (and drags) immediately.
        b = b.focusable(false).focused(false).accept_first_mouse(true);
    }
    let window = b.build()?;

    #[cfg(target_os = "macos")]
    if win.floating() {
        crate::macos::make_panel(&window);
    }

    // Floating panels are hidden, never destroyed: tearing down a webview in a
    // window whose class was swapped to NSPanel crashes WebKit on macOS. The
    // main window is kept alive the same way because it plays key clicks.
    if win.floating() || win == Win::Main {
        let w2 = window.clone();
        let app2 = app.clone();
        window.on_window_event(move |e| {
            if let WindowEvent::CloseRequested { api, .. } = e {
                api.prevent_close();
                if matches!(win, Win::Widget | Win::Indicator) {
                    crate::commands::set_floating(&app2, win, false);
                } else {
                    let _ = w2.hide();
                    st_flag(&app2, win, false);
                    sync_dock(&app2, None);
                }
            }
        });
    } else {
        // Converter and Onboarding are destroyed on close.
        let app2 = app.clone();
        window.on_window_event(move |e| {
            if let WindowEvent::Destroyed = e {
                sync_dock(&app2, Some(win));
            }
        });
    }

    match win {
        Win::Widget | Win::Indicator => {
            let app2 = app.clone();
            let w2 = window.clone();
            window.on_window_event(move |e| match e {
                WindowEvent::Moved(pos) => {
                    let monitor = w2.current_monitor().ok().flatten().and_then(|m| m.name().cloned()).unwrap_or_default();
                    let st = app2.state::<AppState>();
                    // Kept in memory while dragging; the window saves when the drag ends.
                    let mut s = st.settings.write().unwrap();
                    let saved = if win == Win::Widget { &mut s.widget_pos } else { &mut s.indicator_pos };
                    saved.insert(monitor, (pos.x as f64, pos.y as f64));
                }
                _ => {}
            });
        }
        Win::Main => {
            // Recording a shortcut lets every key through: never leave it on
            // once Preferences is closed or in the background.
            let app2 = app.clone();
            window.on_window_event(move |e| match e {
                WindowEvent::Focused(false) | WindowEvent::CloseRequested { .. } => {
                    app2.state::<AppState>().recording.store(false, Ordering::SeqCst);
                }
                _ => {}
            });
        }
        Win::Keyboard => {
            let app2 = app.clone();
            window.on_window_event(move |e| match e {
                WindowEvent::Focused(false) => set_keyboard_focus(&app2, false),
                _ => {}
            });
        }
        _ => {}
    }
    Ok(window)
}

fn st_flag(app: &AppHandle, win: Win, on: bool) {
    let st = app.state::<AppState>();
    match win {
        Win::Widget => st.widget_visible.store(on, Ordering::SeqCst),
        Win::Keyboard => st.keyboard_visible.store(on, Ordering::SeqCst),
        _ => {}
    }
}

fn place(app: &AppHandle, win: Win, window: &WebviewWindow) {
    let Ok(Some(monitor)) = window.primary_monitor().or_else(|_| window.current_monitor()) else { return };
    let scale = monitor.scale_factor();
    let msize = monitor.size();
    let mpos = monitor.position();
    let (w, _) = win.size();
    let menu_bar = if cfg!(target_os = "macos") { 36.0 } else { 16.0 };
    match win {
        Win::Indicator => {
            let st = app.state::<AppState>();
            let saved = monitor.name().and_then(|n| st.settings.read().unwrap().indicator_pos.get(n).copied());
            let (x, y) = saved.unwrap_or((mpos.x as f64 + msize.width as f64 - (w + 16.0) * scale, mpos.y as f64 + menu_bar * scale));
            let _ = window.set_position(PhysicalPosition::new(x, y));
        }
        Win::Widget => {
            let st = app.state::<AppState>();
            let saved = monitor.name().and_then(|n| st.settings.read().unwrap().widget_pos.get(n).copied());
            let (x, y) = saved.unwrap_or((
                mpos.x as f64 + msize.width as f64 - (w + 24.0) * scale,
                mpos.y as f64 + (menu_bar + 64.0) * scale,
            ));
            let _ = window.set_position(PhysicalPosition::new(x, y));
        }
        _ => {}
    }
}

pub fn get_or_build(app: &AppHandle, win: Win) -> Option<WebviewWindow> {
    if let Some(w) = app.get_webview_window(win.label()) {
        return Some(w);
    }
    match build(app, win) {
        Ok(w) => {
            place(app, win, &w);
            Some(w)
        }
        Err(e) => {
            log::error!("cannot open {} window: {e}", win.label());
            None
        }
    }
}

pub fn show(app: &AppHandle, win: Win) {
    let Some(w) = get_or_build(app, win) else { return };
    st_flag(app, win, true);
    if win.floating() {
        #[cfg(target_os = "macos")]
        crate::macos::order_front(&w);
        #[cfg(not(target_os = "macos"))]
        let _ = w.show();
    } else {
        set_dock(app, true);
        #[cfg(target_os = "macos")]
        crate::macos::activate_app();
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
    let _ = app.emit("window-shown", win.label());
}

pub fn hide(app: &AppHandle, win: Win) {
    if let Some(w) = app.get_webview_window(win.label()) {
        let _ = w.hide();
    }
    st_flag(app, win, false);
    sync_dock(app, None);
}

/// Like a menu bar app (Docker, …): the Dock icon is shown only while a regular
/// window is open; with every window closed the app lives on in the menu bar.
pub fn sync_dock(app: &AppHandle, closing: Option<Win>) {
    let open = [Win::Main, Win::Converter, Win::Onboarding].into_iter().any(|win| {
        Some(win) != closing && app.get_webview_window(win.label()).and_then(|w| w.is_visible().ok()).unwrap_or(false)
    });
    set_dock(app, open);
}

fn set_dock(app: &AppHandle, visible: bool) {
    #[cfg(target_os = "macos")]
    {
        let policy = if visible { tauri::ActivationPolicy::Regular } else { tauri::ActivationPolicy::Accessory };
        let _ = app.set_activation_policy(policy);
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (app, visible);
}

pub fn toggle(app: &AppHandle, win: Win) {
    let visible = app.get_webview_window(win.label()).and_then(|w| w.is_visible().ok()).unwrap_or(false);
    if visible {
        hide(app, win);
    } else {
        show(app, win);
    }
}

/// Let the on-screen keyboard's test field take keyboard focus (or give it back).
pub fn set_keyboard_focus(app: &AppHandle, on: bool) {
    let Some(w) = app.get_webview_window(Win::Keyboard.label()) else { return };
    #[cfg(target_os = "macos")]
    crate::macos::allow_key(&w, on);
    #[cfg(not(target_os = "macos"))]
    {
        let _ = w.set_focusable(on);
        if on {
            let _ = w.set_focus();
        }
    }
}

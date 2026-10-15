//! Turns system-wide key events into Bangla text.

use std::sync::atomic::Ordering;

use okkhor_core::Key;
use okkhor_input::{Decision, InputEvent, Modifiers, Special};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::state::AppState;
use crate::windows::{self, Win};

#[derive(Clone, Serialize)]
struct KeyFlash {
    code: &'static str,
    shift: bool,
}

#[derive(Clone, Copy)]
enum Action {
    Toggle,
    Keyboard,
    Converter,
    Widget,
    Taka,
}

fn shortcut_action(st: &AppState, key: Option<Key>, mods: &Modifiers) -> Option<Action> {
    let s = st.settings.read().unwrap();
    let sc = &s.shortcuts;
    let action = [
        (&sc.toggle, Action::Toggle),
        (&sc.keyboard, Action::Keyboard),
        (&sc.converter, Action::Converter),
        (&sc.widget, Action::Widget),
        (&sc.taka, Action::Taka),
    ]
    .into_iter()
    .find(|(s, _)| s.matches(key, mods))
    .map(|(_, a)| a);
    action
}

fn run_action(app: &AppHandle, action: Action) {
    let st = app.state::<AppState>();
    match action {
        Action::Toggle => st.set_mode(app, !st.is_bangla()),
        Action::Taka => {
            st.end_run(app);
            okkhor_input::type_text("৳");
        }
        Action::Keyboard => windows::toggle(app, Win::Keyboard),
        Action::Converter => windows::show(app, Win::Converter),
        Action::Widget => windows::toggle(app, Win::Widget),
    }
}

/// The hook handler. Runs on the hook thread for every key and click.
pub fn handle(app: &AppHandle, ev: InputEvent) -> Decision {
    let st = app.state::<AppState>();
    let (key, special, mods) = match ev {
        InputEvent::MouseDown => {
            st.end_run(app);
            return Decision::Pass;
        }
        InputEvent::Key { key, special, mods } => (key, special, mods),
    };
    if st.recording.load(Ordering::SeqCst) {
        return Decision::Pass;
    }
    if let Some(action) = shortcut_action(&st, key, &mods) {
        let app2 = app.clone();
        let _ = app.run_on_main_thread(move || run_action(&app2, action));
        return Decision::Swallow;
    }
    if let Some(k) = key {
        if st.keyboard_visible.load(Ordering::SeqCst) || st.settings.read().unwrap().sound {
            let _ = app.emit("key", KeyFlash { code: k.code(), shift: mods.shift });
        }
    }
    if !st.is_bangla() {
        return Decision::Pass;
    }
    let key = match (key, special) {
        (Some(k), None) if !mods.any_command() => k,
        _ => {
            st.end_run(app);
            return Decision::Pass;
        }
    };
    if key == Key::Space {
        st.end_run(app);
        return Decision::Pass;
    }
    type_key(app, &st, key, mods.shift, true);
    Decision::Swallow
}

/// Type one Bangla-mode key. `pick` allows 1–4 to choose a visible suggestion.
pub fn type_key(app: &AppHandle, st: &AppState, key: Key, shift: bool, pick: bool) {
    if pick && !shift && st.widget_visible.load(Ordering::SeqCst) {
        if let Some(n) = key.digit_value().filter(|n| (1..=4).contains(n)) {
            let word = st.suggestions.lock().unwrap().get(n as usize - 1).cloned();
            if let Some(word) = word {
                accept(app, st, &word);
                return;
            }
        }
    }
    let edit = st.engine.lock().unwrap().key(key, shift);
    okkhor_input::apply(&edit);
    st.refresh_suggestions(app);
}

pub fn accept(app: &AppHandle, st: &AppState, word: &str) {
    let edit = st.engine.lock().unwrap().accept_suggestion(word);
    okkhor_input::apply(&edit);
    st.dict.lock().unwrap().learn(word);
    st.set_suggestions(app, Vec::new());
}

/// On-screen keyboard: a non-character key.
pub fn virtual_special(app: &AppHandle, st: &AppState, special: Special) {
    st.end_run(app);
    let text = match special {
        Special::Enter => "\n",
        Special::Tab => "\t",
        Special::Backspace => {
            okkhor_input::apply(&okkhor_core::Edit { delete: 1, insert: String::new() });
            return;
        }
        _ => return,
    };
    okkhor_input::type_text(text);
}

/// On-screen keyboard: a character key.
pub fn virtual_key(app: &AppHandle, st: &AppState, key: Key, shift: bool) {
    if key == Key::Space {
        st.end_run(app);
        okkhor_input::type_text(" ");
    } else if st.is_bangla() {
        type_key(app, st, key, shift, false);
    } else {
        st.end_run(app);
        okkhor_input::type_text(&key.latin(shift).to_string());
    }
}

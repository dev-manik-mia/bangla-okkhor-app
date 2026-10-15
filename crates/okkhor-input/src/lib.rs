//! System-wide keyboard capture and text injection.
//!
//! Each platform backend installs a low-level hook that sees every physical
//! key press before the focused app does. The host decides per key whether
//! to let it through or swallow it and inject replacement text instead:
//!
//! - macOS: a Quartz event tap (needs the Accessibility permission).
//! - Windows: `WH_KEYBOARD_LL` / `WH_MOUSE_LL` hooks and `SendInput`.
//! - Linux: grabs the evdev keyboards, re-emits passed keys through a
//!   uinput device and types Bangla through XTest (X11 / XWayland).

use okkhor_core::{Edit, Key};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
use linux as platform;
#[cfg(target_os = "macos")]
use macos as platform;
#[cfg(target_os = "windows")]
use windows as platform;

/// Tag placed on events we inject so our own hook ignores them.
#[cfg_attr(target_os = "linux", allow(dead_code))]
pub(crate) const MAGIC: i64 = 0x4F4B_4B48; // "OKKH"

/// Non-character keys the host cares about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Special {
    Backspace,
    Enter,
    Tab,
    Escape,
    /// Arrows, Home/End, Page Up/Down, forward delete — the caret moved.
    Navigation,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    /// ⌘ on macOS, the Windows / Super key elsewhere.
    pub meta: bool,
}

impl Modifiers {
    pub fn any_command(&self) -> bool {
        self.ctrl || self.alt || self.meta
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEvent {
    /// A key went down (including auto-repeat).
    Key { key: Option<Key>, special: Option<Special>, mods: Modifiers },
    /// Any mouse button went down: the caret may have moved.
    MouseDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Pass,
    Swallow,
}

/// Called on the hook thread for every event. Must return quickly.
pub type Handler = Box<dyn FnMut(InputEvent) -> Decision + Send + 'static>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Permission {
    Granted,
    /// Human-readable explanation of what is missing.
    Missing(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AppInfo {
    /// Display name, e.g. "Microsoft Word".
    pub name: String,
    /// Stable identifier: bundle ID on macOS, executable name elsewhere.
    pub id: String,
}

/// Start the system-wide hook on its own thread. Fails if the permission is
/// missing; call again once it has been granted.
pub fn start(handler: Handler) -> Result<(), String> {
    platform::start(handler)
}

pub fn is_running() -> bool {
    platform::is_running()
}

/// Apply an engine edit to the focused app: backspaces, then text.
pub fn apply(edit: &Edit) {
    if !edit.is_empty() {
        platform::inject(edit.delete, &edit.insert);
    }
}

/// Type text into the focused app.
pub fn type_text(text: &str) {
    platform::inject(0, text);
}

/// Send the platform's Copy shortcut (⌘C / Ctrl+C) to the focused app.
pub fn send_copy() {
    platform::send_shortcut('c');
}

/// Send the platform's Paste shortcut (⌘V / Ctrl+V) to the focused app.
pub fn send_paste() {
    platform::send_shortcut('v');
}

pub fn permission() -> Permission {
    platform::permission()
}

/// Ask the OS for the permission (shows the system prompt where there is one).
pub fn request_permission() {
    platform::request_permission()
}

/// The app that currently has keyboard focus.
pub fn frontmost_app() -> Option<AppInfo> {
    platform::frontmost_app()
}

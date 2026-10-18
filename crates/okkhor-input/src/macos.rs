use std::collections::HashSet;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::sync::Mutex;

use core_foundation::base::TCFType;
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_foundation::string::CFString;
use core_graphics::event::{
    CGEvent, CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
    CallbackResult, EventField,
};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use okkhor_core::Key;

use crate::{AppInfo, Decision, Handler, InputEvent, Modifiers, Permission, Special, MAGIC};

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> bool;
    fn CGEventTapEnable(tap: *mut c_void, enable: bool);
}

static RUNNING: AtomicBool = AtomicBool::new(false);
static TAP_PORT: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

const KC_DELETE: u16 = 51;
const KC_RETURN: u16 = 36;
const KC_KP_ENTER: u16 = 76;
const KC_TAB: u16 = 48;
const KC_ESCAPE: u16 = 53;
const KC_C: u16 = 8;
const KC_V: u16 = 9;

fn special(kc: u16) -> Option<Special> {
    Some(match kc {
        KC_DELETE => Special::Backspace,
        KC_RETURN | KC_KP_ENTER => Special::Enter,
        KC_TAB => Special::Tab,
        KC_ESCAPE => Special::Escape,
        // arrows, home/end, page up/down, forward delete
        123..=126 | 115 | 119 | 116 | 121 | 117 => Special::Navigation,
        _ => return None,
    })
}

pub fn is_running() -> bool {
    RUNNING.load(Ordering::SeqCst)
}

pub fn start(handler: Handler) -> Result<(), String> {
    if is_running() {
        return Ok(());
    }
    if !unsafe { AXIsProcessTrusted() } {
        return Err("Accessibility permission is not granted".into());
    }
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::Builder::new()
        .name("okkhor-event-tap".into())
        .spawn(move || run_tap(handler, tx))
        .map_err(|e| e.to_string())?;
    rx.recv().map_err(|e| e.to_string())?
}

fn run_tap(handler: Handler, ready: std::sync::mpsc::Sender<Result<(), String>>) {
    let handler = Mutex::new(handler);
    // Key-ups for keys whose key-down we swallowed are swallowed too.
    let swallowed: Mutex<HashSet<u16>> = Mutex::new(HashSet::new());
    let tap = CGEventTap::new(
        CGEventTapLocation::Session,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::Default,
        vec![CGEventType::KeyDown, CGEventType::KeyUp, CGEventType::LeftMouseDown, CGEventType::RightMouseDown, CGEventType::OtherMouseDown],
        move |_proxy, etype, event| {
            match etype {
                CGEventType::TapDisabledByTimeout | CGEventType::TapDisabledByUserInput => {
                    let port = TAP_PORT.load(Ordering::SeqCst);
                    if !port.is_null() {
                        unsafe { CGEventTapEnable(port, true) };
                    }
                    return CallbackResult::Keep;
                }
                CGEventType::LeftMouseDown | CGEventType::RightMouseDown | CGEventType::OtherMouseDown => {
                    (handler.lock().unwrap())(InputEvent::MouseDown);
                    return CallbackResult::Keep;
                }
                _ => {}
            }
            if event.get_integer_value_field(EventField::EVENT_SOURCE_USER_DATA) == MAGIC {
                return CallbackResult::Keep;
            }
            let kc = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as u16;
            if matches!(etype, CGEventType::KeyUp) {
                return if swallowed.lock().unwrap().remove(&kc) { CallbackResult::Drop } else { CallbackResult::Keep };
            }
            let flags = event.get_flags();
            let mods = Modifiers {
                shift: flags.contains(CGEventFlags::CGEventFlagShift),
                ctrl: flags.contains(CGEventFlags::CGEventFlagControl),
                alt: flags.contains(CGEventFlags::CGEventFlagAlternate),
                meta: flags.contains(CGEventFlags::CGEventFlagCommand),
            };
            let ev = InputEvent::Key { key: Key::from_mac_keycode(kc), special: special(kc), mods };
            match (handler.lock().unwrap())(ev) {
                Decision::Pass => CallbackResult::Keep,
                Decision::Swallow => {
                    swallowed.lock().unwrap().insert(kc);
                    CallbackResult::Drop
                }
            }
        },
    );
    let tap = match tap {
        Ok(t) => t,
        Err(()) => {
            let _ = ready.send(Err("Could not create the keyboard event tap".into()));
            return;
        }
    };
    TAP_PORT.store(tap.mach_port().as_concrete_TypeRef() as *mut c_void, Ordering::SeqCst);
    let source = match tap.mach_port().create_runloop_source(0) {
        Ok(s) => s,
        Err(()) => {
            let _ = ready.send(Err("Could not attach the event tap to a run loop".into()));
            return;
        }
    };
    CFRunLoop::get_current().add_source(&source, unsafe { kCFRunLoopCommonModes });
    tap.enable();
    RUNNING.store(true, Ordering::SeqCst);
    let _ = ready.send(Ok(()));
    CFRunLoop::run_current();
    RUNNING.store(false, Ordering::SeqCst);
    TAP_PORT.store(std::ptr::null_mut(), Ordering::SeqCst);
}

fn source() -> Option<CGEventSource> {
    CGEventSource::new(CGEventSourceStateID::Private).ok()
}

fn post_key(src: &CGEventSource, keycode: u16, flags: CGEventFlags) {
    for down in [true, false] {
        if let Ok(ev) = CGEvent::new_keyboard_event(src.clone(), keycode, down) {
            ev.set_flags(flags);
            ev.set_integer_value_field(EventField::EVENT_SOURCE_USER_DATA, MAGIC);
            ev.post(CGEventTapLocation::Session);
        }
    }
}

pub fn inject(delete: usize, text: &str) {
    let Some(src) = source() else { return };
    for _ in 0..delete {
        post_key(&src, KC_DELETE, CGEventFlags::CGEventFlagNull);
    }
    // CGEventKeyboardSetUnicodeString accepts at most 20 UTF-16 units per event.
    let units: Vec<u16> = text.encode_utf16().collect();
    let mut start = 0;
    while start < units.len() {
        let mut end = (start + 20).min(units.len());
        // don't split a surrogate pair
        if end < units.len() && (0xDC00..=0xDFFF).contains(&units[end]) {
            end -= 1;
        }
        for down in [true, false] {
            if let Ok(ev) = CGEvent::new_keyboard_event(src.clone(), 0, down) {
                ev.set_flags(CGEventFlags::CGEventFlagNull);
                ev.set_string_from_utf16_unchecked(&units[start..end]);
                ev.set_integer_value_field(EventField::EVENT_SOURCE_USER_DATA, MAGIC);
                ev.post(CGEventTapLocation::Session);
            }
        }
        start = end;
    }
}

pub fn send_shortcut(letter: char) {
    let Some(src) = source() else { return };
    let kc = if letter == 'v' { KC_V } else { KC_C };
    post_key(&src, kc, CGEventFlags::CGEventFlagCommand);
}

pub fn permission() -> Permission {
    if unsafe { AXIsProcessTrusted() } {
        Permission::Granted
    } else {
        Permission::Missing(if in_app_bundle() {
            "Allow BanglaOkkhor in System Settings → Privacy & Security → Accessibility.".into()
        } else {
            "Running outside BanglaOkkhor.app: allow the app that launched it (your terminal or editor) in System Settings → Privacy & Security → Accessibility.".into()
        })
    }
}

pub fn in_app_bundle() -> bool {
    std::env::current_exe().is_ok_and(|p| p.to_string_lossy().contains(".app/Contents/MacOS/"))
}

pub fn request_permission() {
    let key = CFString::new("AXTrustedCheckOptionPrompt");
    let opts = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), CFBoolean::true_value().as_CFType())]);
    unsafe { AXIsProcessTrustedWithOptions(opts.as_concrete_TypeRef()) };
}

#[allow(unused_unsafe)]
pub fn frontmost_app() -> Option<AppInfo> {
    use objc2_app_kit::NSWorkspace;
    unsafe {
        let ws = NSWorkspace::sharedWorkspace();
        let app = ws.frontmostApplication()?;
        let name = app.localizedName().map(|s| s.to_string()).unwrap_or_default();
        let id = app.bundleIdentifier().map(|s| s.to_string()).unwrap_or_else(|| name.clone());
        Some(AppInfo { name, id })
    }
}

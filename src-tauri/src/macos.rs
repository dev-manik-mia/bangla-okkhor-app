//! macOS: turn floating windows into non-activating panels, so clicking the
//! on-screen keyboard or the widget never takes focus from the app the user
//! is typing in.

use std::collections::HashSet;
use std::ffi::c_void;
use std::sync::{Mutex, OnceLock};

use objc2::runtime::{AnyClass, AnyObject, Bool, ClassBuilder, Sel};
use objc2::{msg_send, sel};
use tauri::WebviewWindow;

/// Panels currently allowed to become the key window (the keyboard's test field).
static ALLOW_KEY: Mutex<Option<HashSet<usize>>> = Mutex::new(None);

const NS_WINDOW_STYLE_MASK_NONACTIVATING_PANEL: usize = 1 << 7;
const NS_COLLECTION_CAN_JOIN_ALL_SPACES: usize = 1 << 0;
const NS_COLLECTION_FULL_SCREEN_AUXILIARY: usize = 1 << 8;
const NS_FLOATING_WINDOW_LEVEL: isize = 3;

extern "C-unwind" fn can_become_key(this: &AnyObject, _: Sel) -> Bool {
    let ptr = this as *const AnyObject as usize;
    Bool::new(ALLOW_KEY.lock().unwrap().as_ref().is_some_and(|s| s.contains(&ptr)))
}

extern "C-unwind" fn no(_: &AnyObject, _: Sel) -> Bool {
    Bool::NO
}

fn panel_class() -> &'static AnyClass {
    static CLASS: OnceLock<usize> = OnceLock::new();
    let ptr = *CLASS.get_or_init(|| {
        let superclass = AnyClass::get(c"NSPanel").expect("NSPanel exists");
        let mut b = ClassBuilder::new(c"BanglaOkkhorPanel", superclass).expect("register panel class once");
        unsafe {
            b.add_method(sel!(canBecomeKeyWindow), can_become_key as extern "C-unwind" fn(_, _) -> _);
            b.add_method(sel!(canBecomeMainWindow), no as extern "C-unwind" fn(_, _) -> _);
        }
        b.register() as *const AnyClass as usize
    });
    unsafe { &*(ptr as *const AnyClass) }
}

extern "C" {
    fn object_setClass(obj: *mut c_void, cls: *const c_void) -> *const c_void;
}

fn ns_window(w: &WebviewWindow) -> Option<*mut AnyObject> {
    w.ns_window().ok().map(|p| p as *mut AnyObject)
}

pub fn make_panel(w: &WebviewWindow) {
    let w2 = w.clone();
    let _ = w.run_on_main_thread(move || {
        let Some(win) = ns_window(&w2) else { return };
        unsafe {
            object_setClass(win as *mut c_void, panel_class() as *const AnyClass as *const c_void);
            let mask: usize = msg_send![win, styleMask];
            let _: () = msg_send![win, setStyleMask: mask | NS_WINDOW_STYLE_MASK_NONACTIVATING_PANEL];
            let _: () = msg_send![win, setFloatingPanel: true];
            let _: () = msg_send![win, setBecomesKeyOnlyIfNeeded: true];
            let _: () = msg_send![win, setHidesOnDeactivate: false];
            let _: () = msg_send![win, setLevel: NS_FLOATING_WINDOW_LEVEL];
            let _: () = msg_send![
                win,
                setCollectionBehavior: NS_COLLECTION_CAN_JOIN_ALL_SPACES | NS_COLLECTION_FULL_SCREEN_AUXILIARY
            ];
        }
    });
}

/// Show a panel without activating the app.
pub fn order_front(w: &WebviewWindow) {
    let w2 = w.clone();
    let _ = w.run_on_main_thread(move || {
        if let Some(win) = ns_window(&w2) {
            unsafe {
                let _: () = msg_send![win, orderFrontRegardless];
            }
        }
    });
}

pub fn allow_key(w: &WebviewWindow, on: bool) {
    let w2 = w.clone();
    let _ = w.run_on_main_thread(move || {
        let Some(win) = ns_window(&w2) else { return };
        {
            let mut set = ALLOW_KEY.lock().unwrap();
            let set = set.get_or_insert_with(HashSet::new);
            if on {
                set.insert(win as usize);
            } else {
                set.remove(&(win as usize));
            }
        }
        if on {
            unsafe {
                let _: () = msg_send![win, makeKeyWindow];
            }
        }
    });
}

/// Bring the app forward (it has no Dock icon) so a regular window shows on top.
pub fn activate_app() {
    let Some(cls) = AnyClass::get(c"NSApplication") else { return };
    unsafe {
        let app: *mut AnyObject = msg_send![cls, sharedApplication];
        let _: () = msg_send![app, activateIgnoringOtherApps: true];
    }
}

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex, OnceLock};

use enigo::{Direction, Enigo, Keyboard, Settings};
use evdev::uinput::VirtualDevice;
use evdev::{AttributeSet, Device, EventType, KeyCode};
use okkhor_core::Key;

use crate::{AppInfo, Decision, Handler, InputEvent, Modifiers, Permission, Special};

const VIRTUAL_NAME: &str = "BanglaOkkhor passthrough keyboard";

static RUNNING: AtomicBool = AtomicBool::new(false);

enum Cmd {
    Inject(usize, String),
    Shortcut(char),
}

/// Text goes out through XTest on one thread so backspaces and text stay in order.
fn injector() -> &'static Mutex<Sender<Cmd>> {
    static TX: OnceLock<Mutex<Sender<Cmd>>> = OnceLock::new();
    TX.get_or_init(|| {
        let (tx, rx) = channel::<Cmd>();
        std::thread::Builder::new()
            .name("okkhor-injector".into())
            .spawn(move || {
                let mut enigo = match Enigo::new(&Settings::default()) {
                    Ok(e) => e,
                    Err(e) => {
                        log::error!("cannot connect to the X server for text injection: {e}");
                        return;
                    }
                };
                for cmd in rx {
                    match cmd {
                        Cmd::Inject(delete, text) => {
                            for _ in 0..delete {
                                let _ = enigo.key(enigo::Key::Backspace, Direction::Click);
                            }
                            if !text.is_empty() {
                                let _ = enigo.text(&text);
                            }
                        }
                        Cmd::Shortcut(c) => {
                            let _ = enigo.key(enigo::Key::Control, Direction::Press);
                            let _ = enigo.key(enigo::Key::Unicode(c), Direction::Click);
                            let _ = enigo.key(enigo::Key::Control, Direction::Release);
                        }
                    }
                }
            })
            .expect("spawn injector thread");
        Mutex::new(tx)
    })
}

pub fn is_running() -> bool {
    RUNNING.load(Ordering::SeqCst)
}

fn is_keyboard(dev: &Device) -> bool {
    if dev.name().is_some_and(|n| n == VIRTUAL_NAME) {
        return false;
    }
    dev.supported_keys()
        .is_some_and(|k| k.contains(KeyCode::KEY_A) && k.contains(KeyCode::KEY_SPACE) && k.contains(KeyCode::KEY_ENTER))
}

fn special(code: KeyCode) -> Option<Special> {
    Some(match code {
        KeyCode::KEY_BACKSPACE => Special::Backspace,
        KeyCode::KEY_ENTER | KeyCode::KEY_KPENTER => Special::Enter,
        KeyCode::KEY_TAB => Special::Tab,
        KeyCode::KEY_ESC => Special::Escape,
        KeyCode::KEY_LEFT | KeyCode::KEY_RIGHT | KeyCode::KEY_UP | KeyCode::KEY_DOWN | KeyCode::KEY_HOME
        | KeyCode::KEY_END | KeyCode::KEY_PAGEUP | KeyCode::KEY_PAGEDOWN | KeyCode::KEY_DELETE => Special::Navigation,
        _ => return None,
    })
}

pub fn start(handler: Handler) -> Result<(), String> {
    if is_running() {
        return Ok(());
    }
    if let Permission::Missing(why) = permission() {
        return Err(why);
    }
    let keyboards: Vec<Device> = evdev::enumerate().map(|(_, d)| d).filter(is_keyboard).collect();
    if keyboards.is_empty() {
        return Err("No keyboard found under /dev/input".into());
    }
    let handler = Arc::new(Mutex::new(handler));
    for mut dev in keyboards {
        let keys: AttributeSet<KeyCode> = dev.supported_keys().map(|k| k.iter().collect()).unwrap_or_default();
        let mut virt = VirtualDevice::builder()
            .and_then(|b| b.name(VIRTUAL_NAME).with_keys(&keys))
            .and_then(|b| b.build())
            .map_err(|e| format!("cannot create uinput device: {e}"))?;
        dev.grab().map_err(|e| format!("cannot grab {}: {e}", dev.name().unwrap_or("keyboard")))?;
        let handler = handler.clone();
        std::thread::Builder::new()
            .name("okkhor-evdev".into())
            .spawn(move || {
                let mut held: HashSet<KeyCode> = HashSet::new();
                let mut swallowed: HashSet<KeyCode> = HashSet::new();
                loop {
                    let events: Vec<evdev::InputEvent> = match dev.fetch_events() {
                        Ok(evs) => evs.collect(),
                        Err(e) => {
                            log::error!("keyboard read failed: {e}");
                            break;
                        }
                    };
                    let mut out = Vec::with_capacity(events.len());
                    for ev in events {
                        if ev.event_type() == EventType::SYNCHRONIZATION {
                            continue;
                        }
                        if ev.event_type() != EventType::KEY {
                            out.push(ev);
                            continue;
                        }
                        let code = KeyCode::new(ev.code());
                        match ev.value() {
                            0 => {
                                held.remove(&code);
                                if !swallowed.remove(&code) {
                                    out.push(ev);
                                }
                            }
                            _ => {
                                held.insert(code);
                                // buttons (BTN_MISC and up) are not typing keys
                                if code.code() >= 0x100 {
                                    out.push(ev);
                                    continue;
                                }
                                let h = |a: KeyCode, b: KeyCode| held.contains(&a) || held.contains(&b);
                                let mods = Modifiers {
                                    shift: h(KeyCode::KEY_LEFTSHIFT, KeyCode::KEY_RIGHTSHIFT),
                                    ctrl: h(KeyCode::KEY_LEFTCTRL, KeyCode::KEY_RIGHTCTRL),
                                    alt: h(KeyCode::KEY_LEFTALT, KeyCode::KEY_RIGHTALT),
                                    meta: h(KeyCode::KEY_LEFTMETA, KeyCode::KEY_RIGHTMETA),
                                };
                                let key = Key::from_scancode(ev.code() as u32);
                                let decision = (handler.lock().unwrap())(InputEvent::Key { key, special: special(code), mods });
                                match decision {
                                    Decision::Pass => out.push(ev),
                                    Decision::Swallow => {
                                        swallowed.insert(code);
                                    }
                                }
                            }
                        }
                    }
                    if !out.is_empty() {
                        if let Err(e) = virt.emit(&out) {
                            log::error!("uinput emit failed: {e}");
                        }
                    }
                }
            })
            .map_err(|e| e.to_string())?;
    }
    RUNNING.store(true, Ordering::SeqCst);
    Ok(())
}

pub fn inject(delete: usize, text: &str) {
    let _ = injector().lock().unwrap().send(Cmd::Inject(delete, text.to_string()));
}

pub fn send_shortcut(letter: char) {
    let _ = injector().lock().unwrap().send(Cmd::Shortcut(letter));
}

fn can_open(path: &str, write: bool) -> bool {
    std::fs::OpenOptions::new().read(true).write(write).open(path).is_ok()
}

pub fn permission() -> Permission {
    let readable = std::fs::read_dir("/dev/input")
        .map(|d| {
            d.filter_map(|e| e.ok())
                .filter(|e| e.file_name().to_string_lossy().starts_with("event"))
                .any(|e| can_open(&e.path().to_string_lossy(), false))
        })
        .unwrap_or(false);
    if !readable {
        return Permission::Missing(
            "Add your user to the 'input' group: sudo usermod -aG input $USER, then log out and back in.".into(),
        );
    }
    if !can_open("/dev/uinput", true) {
        return Permission::Missing(
            "Allow access to /dev/uinput (install the BanglaOkkhor udev rule, or: sudo modprobe uinput).".into(),
        );
    }
    Permission::Granted
}

pub fn request_permission() {}

pub fn frontmost_app() -> Option<AppInfo> {
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::{AtomEnum, ConnectionExt};

    let (conn, screen) = x11rb::connect(None).ok()?;
    let root = conn.setup().roots.get(screen)?.root;
    let atom = |name: &[u8]| conn.intern_atom(false, name).ok()?.reply().ok().map(|r| r.atom);
    let active = atom(b"_NET_ACTIVE_WINDOW")?;
    let reply = conn.get_property(false, root, active, AtomEnum::WINDOW, 0, 1).ok()?.reply().ok()?;
    let window = reply.value32()?.next()?;
    if window == 0 {
        return None;
    }
    let class = conn.get_property(false, window, AtomEnum::WM_CLASS, AtomEnum::STRING, 0, 256).ok()?.reply().ok()?;
    // WM_CLASS is "instance\0Class\0"
    let mut parts = class.value.split(|b| *b == 0).filter(|p| !p.is_empty());
    let instance = String::from_utf8_lossy(parts.next()?).to_string();
    let name = parts.next().map(|p| String::from_utf8_lossy(p).to_string()).unwrap_or_else(|| instance.clone());
    Some(AppInfo { name, id: instance.to_lowercase() })
}

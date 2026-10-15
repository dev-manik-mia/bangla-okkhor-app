use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

use okkhor_core::Key;
use windows_sys::Win32::Foundation::{CloseHandle, HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE,
    VIRTUAL_KEY, VK_BACK, VK_CONTROL, VK_DELETE, VK_DOWN, VK_END, VK_ESCAPE, VK_HOME, VK_LEFT, VK_LWIN, VK_MENU,
    VK_NEXT, VK_PRIOR, VK_RETURN, VK_RIGHT, VK_RWIN, VK_SHIFT, VK_TAB, VK_UP,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetForegroundWindow, GetMessageW, GetWindowThreadProcessId,
    SetWindowsHookExW, TranslateMessage, HC_ACTION, KBDLLHOOKSTRUCT, LLKHF_EXTENDED, MSG, WH_KEYBOARD_LL,
    WH_MOUSE_LL, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_MBUTTONDOWN, WM_RBUTTONDOWN, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

use crate::{AppInfo, Decision, Handler, InputEvent, Modifiers, Permission, Special, MAGIC};

static RUNNING: AtomicBool = AtomicBool::new(false);
static HANDLER: OnceLock<Mutex<Handler>> = OnceLock::new();
static SWALLOWED: Mutex<Option<HashSet<u32>>> = Mutex::new(None);

pub fn is_running() -> bool {
    RUNNING.load(Ordering::SeqCst)
}

pub fn start(handler: Handler) -> Result<(), String> {
    if is_running() {
        return Ok(());
    }
    if HANDLER.set(Mutex::new(handler)).is_err() {
        return Err("hook handler already installed".into());
    }
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::Builder::new()
        .name("okkhor-keyboard-hook".into())
        .spawn(move || unsafe {
            let hinst = GetModuleHandleW(std::ptr::null());
            let kb = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), hinst, 0);
            let mouse = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), hinst, 0);
            if kb.is_null() {
                let _ = tx.send(Err("SetWindowsHookExW(WH_KEYBOARD_LL) failed".to_string()));
                return;
            }
            let _ = mouse;
            RUNNING.store(true, Ordering::SeqCst);
            let _ = tx.send(Ok(()));
            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            RUNNING.store(false, Ordering::SeqCst);
        })
        .map_err(|e| e.to_string())?;
    rx.recv().map_err(|e| e.to_string())?
}

fn down(vk: VIRTUAL_KEY) -> bool {
    (unsafe { GetAsyncKeyState(vk as i32) } as u16 & 0x8000) != 0
}

fn special(vk: u32) -> Option<Special> {
    let vk = vk as VIRTUAL_KEY;
    Some(match vk {
        VK_BACK => Special::Backspace,
        VK_RETURN => Special::Enter,
        VK_TAB => Special::Tab,
        VK_ESCAPE => Special::Escape,
        VK_LEFT | VK_RIGHT | VK_UP | VK_DOWN | VK_HOME | VK_END | VK_PRIOR | VK_NEXT | VK_DELETE => Special::Navigation,
        _ => return None,
    })
}

unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code != HC_ACTION as i32 {
        return CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam);
    }
    let kb = &*(lparam as *const KBDLLHOOKSTRUCT);
    if kb.dwExtraInfo == MAGIC as usize {
        return CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam);
    }
    let msg = wparam as u32;
    let mut swallowed = SWALLOWED.lock().unwrap();
    let set = swallowed.get_or_insert_with(HashSet::new);
    if msg == WM_KEYUP || msg == WM_SYSKEYUP {
        if set.remove(&kb.vkCode) {
            return 1;
        }
        return CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam);
    }
    if msg != WM_KEYDOWN && msg != WM_SYSKEYDOWN {
        return CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam);
    }
    let extended = kb.flags & LLKHF_EXTENDED != 0;
    let key = if extended { None } else { Key::from_scancode(kb.scanCode) };
    let mods = Modifiers {
        shift: down(VK_SHIFT),
        ctrl: down(VK_CONTROL),
        alt: down(VK_MENU),
        meta: down(VK_LWIN) || down(VK_RWIN),
    };
    let ev = InputEvent::Key { key, special: special(kb.vkCode), mods };
    drop(swallowed);
    let decision = match HANDLER.get() {
        Some(h) => (h.lock().unwrap())(ev),
        None => Decision::Pass,
    };
    match decision {
        Decision::Pass => CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam),
        Decision::Swallow => {
            SWALLOWED.lock().unwrap().get_or_insert_with(HashSet::new).insert(kb.vkCode);
            1
        }
    }
}

unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 && matches!(wparam as u32, WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN) {
        if let Some(h) = HANDLER.get() {
            (h.lock().unwrap())(InputEvent::MouseDown);
        }
    }
    CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam)
}

fn key_input(vk: VIRTUAL_KEY, scan: u16, flags: u32) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT { wVk: vk, wScan: scan, dwFlags: flags, time: 0, dwExtraInfo: MAGIC as usize },
        },
    }
}

fn send(inputs: &[INPUT]) {
    if !inputs.is_empty() {
        unsafe { SendInput(inputs.len() as u32, inputs.as_ptr(), std::mem::size_of::<INPUT>() as i32) };
    }
}

pub fn inject(delete: usize, text: &str) {
    let mut inputs = Vec::new();
    for _ in 0..delete {
        inputs.push(key_input(VK_BACK, 0, 0));
        inputs.push(key_input(VK_BACK, 0, KEYEVENTF_KEYUP));
    }
    for unit in text.encode_utf16() {
        inputs.push(key_input(0, unit, KEYEVENTF_UNICODE));
        inputs.push(key_input(0, unit, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP));
    }
    send(&inputs);
}

pub fn send_shortcut(letter: char) {
    let vk = letter.to_ascii_uppercase() as VIRTUAL_KEY;
    send(&[
        key_input(VK_CONTROL, 0, 0),
        key_input(vk, 0, 0),
        key_input(vk, 0, KEYEVENTF_KEYUP),
        key_input(VK_CONTROL, 0, KEYEVENTF_KEYUP),
    ]);
}

pub fn permission() -> Permission {
    Permission::Granted
}

pub fn request_permission() {}

pub fn frontmost_app() -> Option<AppInfo> {
    unsafe {
        let hwnd: HWND = GetForegroundWindow();
        if hwnd.is_null() {
            return None;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, &mut pid);
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if process.is_null() {
            return None;
        }
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(process, PROCESS_NAME_WIN32, buf.as_mut_ptr(), &mut len);
        CloseHandle(process);
        if ok == 0 {
            return None;
        }
        let path = String::from_utf16_lossy(&buf[..len as usize]);
        let exe = path.rsplit('\\').next().unwrap_or(&path).to_string();
        let stem = exe.strip_suffix(".exe").or_else(|| exe.strip_suffix(".EXE")).unwrap_or(&exe).to_string();
        Some(AppInfo { name: pretty_name(&stem), id: stem.to_lowercase() })
    }
}

fn pretty_name(stem: &str) -> String {
    match stem.to_lowercase().as_str() {
        "winword" => "Microsoft Word",
        "excel" => "Microsoft Excel",
        "powerpnt" => "Microsoft PowerPoint",
        "chrome" => "Google Chrome",
        "msedge" => "Microsoft Edge",
        "firefox" => "Firefox",
        "notepad" => "Notepad",
        "explorer" => "File Explorer",
        _ => return stem.to_string(),
    }
    .to_string()
}

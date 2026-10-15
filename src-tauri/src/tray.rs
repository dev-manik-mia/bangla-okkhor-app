//! The menu bar / system tray icon and its menu.

use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{AppHandle, Manager, Wry};

use crate::state::AppState;
use crate::windows::{self, Win};

const TRAY_ID: &str = "okkhor";

#[cfg(target_os = "macos")]
const BN_ICON: &[u8] = include_bytes!("../icons/tray/bn-template.png");
#[cfg(target_os = "macos")]
const EN_ICON: &[u8] = include_bytes!("../icons/tray/en-template.png");
#[cfg(not(target_os = "macos"))]
const BN_ICON: &[u8] = include_bytes!("../icons/tray/bn.png");
#[cfg(not(target_os = "macos"))]
const EN_ICON: &[u8] = include_bytes!("../icons/tray/en.png");

struct Labels {
    bijoy: &'static str,
    avro: &'static str,
    english: &'static str,
    widget: &'static str,
    keyboard: &'static str,
    converter: &'static str,
    prefs: &'static str,
    setup: &'static str,
    quit: &'static str,
}

fn labels(lang: &str) -> Labels {
    if lang == "bn" {
        Labels {
            bijoy: "বাংলা — বিজয়৭১",
            avro: "বাংলা — অভ্র ফোনেটিক",
            english: "ইংরেজি — ইউ.এস.",
            widget: "ভাসমান উইজেট দেখান",
            keyboard: "অন-স্ক্রিন কিবোর্ড দেখান",
            converter: "ইউনিকোড ↔ এএনএসআই কনভার্টার…",
            prefs: "পছন্দসমূহ…",
            setup: "সেটআপ গাইড…",
            quit: "বাংলাঅক্ষর বন্ধ করুন",
        }
    } else {
        Labels {
            bijoy: "Bangla — Bijoy71",
            avro: "Bangla — Avro Phonetic",
            english: "English — U.S.",
            widget: "Show Floating Widget",
            keyboard: "Show On-screen Keyboard",
            converter: "Unicode ↔ ANSI Converter…",
            prefs: "Preferences…",
            setup: "Setup Guide…",
            quit: "Quit BanglaOkkhor",
        }
    }
}

fn build_menu(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let st = app.state::<AppState>();
    let s = st.settings();
    let l = labels(&s.lang);
    let bn = st.is_bangla();
    let avro = s.layout == okkhor_core::Layout::Avro;
    let sc = &s.shortcuts;
    let acc = |sh: &crate::settings::Shortcut| sh.accelerator();
    let menu = Menu::with_items(
        app,
        &[
            &CheckMenuItem::with_id(app, "input-bijoy", l.bijoy, true, bn && !avro, if avro { None } else { acc(&sc.toggle) })?,
            &CheckMenuItem::with_id(app, "input-avro", l.avro, true, bn && avro, if avro { acc(&sc.toggle) } else { None })?,
            &CheckMenuItem::with_id(app, "input-en", l.english, true, !bn, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "widget", l.widget, true, acc(&sc.widget))?,
            &MenuItem::with_id(app, "keyboard", l.keyboard, true, acc(&sc.keyboard))?,
            &MenuItem::with_id(app, "converter", l.converter, true, acc(&sc.converter))?,
            &MenuItem::with_id(app, "prefs", l.prefs, true, Some("CmdOrCtrl+,"))?,
            &MenuItem::with_id(app, "setup", l.setup, true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "quit", l.quit, true, None::<&str>)?,
        ],
    )?;
    Ok(menu)
}

fn icon(bangla: bool) -> Image<'static> {
    Image::from_bytes(if bangla { BN_ICON } else { EN_ICON }).expect("valid tray icon")
}

pub fn create(app: &AppHandle) -> tauri::Result<TrayIcon> {
    let bangla = app.state::<AppState>().is_bangla();
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon(bangla))
        .icon_as_template(cfg!(target_os = "macos"))
        .tooltip("BanglaOkkhor")
        .menu(&build_menu(app)?)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            let st = app.state::<AppState>();
            match event.id().as_ref() {
                "input-bijoy" => st.set_input(app, "bijoy"),
                "input-avro" => st.set_input(app, "avro"),
                "input-en" => st.set_input(app, "en"),
                "widget" => windows::toggle(app, Win::Widget),
                "keyboard" => windows::show(app, Win::Keyboard),
                "converter" => windows::show(app, Win::Converter),
                "prefs" => windows::show(app, Win::Main),
                "setup" => windows::show(app, Win::Onboarding),
                "quit" => {
                    st.save_words();
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)
}

/// Refresh the icon and menu after the mode, language or shortcuts change.
pub fn update(app: &AppHandle) {
    let app2 = app.clone();
    let _ = app.run_on_main_thread(move || {
        let Some(tray) = app2.tray_by_id(TRAY_ID) else { return };
        let bangla = app2.state::<AppState>().is_bangla();
        let _ = tray.set_icon(Some(icon(bangla)));
        let _ = tray.set_icon_as_template(cfg!(target_os = "macos"));
        if let Ok(menu) = build_menu(&app2) {
            let _ = tray.set_menu(Some(menu));
        }
    });
}

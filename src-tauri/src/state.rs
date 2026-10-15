use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Mutex, RwLock};

use okkhor_core::{Dictionary, Encoding, Engine, Layout};
use okkhor_input::AppInfo;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::settings::Settings;

/// The app the user is typing into (never BanglaOkkhor itself).
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    pub name: String,
    pub id: String,
    /// A desktop word processor, where legacy ANSI output makes sense.
    pub word_processor: bool,
    pub encoding: Encoding,
}

pub fn is_word_processor(id: &str) -> bool {
    let id = id.to_lowercase();
    ["com.microsoft.word", "winword", "libreoffice", "soffice", "com.apple.iwork.pages", "wps", "abiword", "writer"]
        .iter()
        .any(|w| id.contains(w))
}

pub struct AppState {
    pub settings: RwLock<Settings>,
    pub engine: Mutex<Engine>,
    pub dict: Mutex<Dictionary>,
    pub bangla: AtomicBool,
    pub suggestions: Mutex<Vec<String>>,
    pub target: Mutex<Target>,
    /// Preferences is recording a shortcut: let every key through.
    pub recording: AtomicBool,
    pub widget_visible: AtomicBool,
    pub keyboard_visible: AtomicBool,
    learned_since_save: AtomicU32,
    config_dir: PathBuf,
}

impl AppState {
    pub fn new(config_dir: PathBuf) -> Self {
        let settings = Settings::load(&config_dir.join("settings.json"));
        let learned: HashMap<String, u32> = std::fs::read_to_string(config_dir.join("words.json"))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        let mut engine = Engine::new(settings.engine_options());
        engine.layout = settings.layout;
        let bangla = settings.start_in != "en";
        AppState {
            settings: RwLock::new(settings),
            engine: Mutex::new(engine),
            dict: Mutex::new(Dictionary::with_learned(learned)),
            bangla: AtomicBool::new(bangla),
            suggestions: Mutex::new(Vec::new()),
            target: Mutex::new(Target::default()),
            recording: AtomicBool::new(false),
            widget_visible: AtomicBool::new(false),
            keyboard_visible: AtomicBool::new(false),
            learned_since_save: AtomicU32::new(0),
            config_dir,
        }
    }

    pub fn settings(&self) -> Settings {
        self.settings.read().unwrap().clone()
    }

    pub fn save_settings(&self) {
        self.settings.read().unwrap().save(&self.config_dir.join("settings.json"));
    }

    pub fn save_words(&self) {
        let dict = self.dict.lock().unwrap();
        if let Ok(s) = serde_json::to_string(dict.learned()) {
            let _ = std::fs::create_dir_all(&self.config_dir);
            let _ = std::fs::write(self.config_dir.join("words.json"), s);
        }
        self.learned_since_save.store(0, Ordering::SeqCst);
    }

    pub fn is_bangla(&self) -> bool {
        self.bangla.load(Ordering::SeqCst)
    }

    /// End the composition run, learning the word that was being typed.
    pub fn end_run(&self, app: &AppHandle) {
        let word = {
            let mut engine = self.engine.lock().unwrap();
            let w = engine.current_word().to_string();
            engine.reset();
            w
        };
        if word.chars().count() >= 2 {
            self.dict.lock().unwrap().learn(&word);
            if self.learned_since_save.fetch_add(1, Ordering::SeqCst) >= 20 {
                self.save_words();
            }
        }
        self.set_suggestions(app, Vec::new());
    }

    pub fn set_suggestions(&self, app: &AppHandle, list: Vec<String>) {
        let mut cur = self.suggestions.lock().unwrap();
        if *cur != list {
            *cur = list.clone();
            let _ = app.emit("suggestions", list);
        }
    }

    pub fn refresh_suggestions(&self, app: &AppHandle) {
        let list = if self.settings.read().unwrap().sugg {
            let word = self.engine.lock().unwrap().current_word().to_string();
            self.dict.lock().unwrap().suggest(&word, 4)
        } else {
            Vec::new()
        };
        self.set_suggestions(app, list);
    }

    pub fn set_mode(&self, app: &AppHandle, bangla: bool) {
        self.end_run(app);
        self.bangla.store(bangla, Ordering::SeqCst);
        let _ = app.emit("mode", bangla);
        crate::tray::update(app);
    }

    /// Pick one of the three input sources: "bijoy", "avro" or "en".
    pub fn set_input(&self, app: &AppHandle, input: &str) {
        let layout = match input {
            "bijoy" => Layout::Bijoy,
            "avro" => Layout::Avro,
            _ => return self.set_mode(app, false),
        };
        let changed = {
            let mut s = self.settings.write().unwrap();
            let changed = s.layout != layout;
            s.layout = layout;
            changed
        };
        if changed {
            self.save_settings();
            self.engine.lock().unwrap().layout = layout;
            let _ = app.emit("settings", self.settings());
        }
        self.set_mode(app, true);
    }

    /// Called when focus moves to another app.
    pub fn set_target(&self, app: &AppHandle, info: AppInfo) {
        let encoding = {
            let settings = self.settings.read().unwrap();
            settings.encodings.get(&info.id).copied().unwrap_or_default()
        };
        let target = Target { word_processor: is_word_processor(&info.id), name: info.name, id: info.id, encoding };
        self.end_run(app);
        self.engine.lock().unwrap().encoding = if target.word_processor { encoding } else { Encoding::Unicode };
        *self.target.lock().unwrap() = target.clone();
        let _ = app.emit("target", target);
    }

    pub fn set_encoding(&self, app: &AppHandle, encoding: Encoding) {
        let target = {
            let mut t = self.target.lock().unwrap();
            t.encoding = encoding;
            t.clone()
        };
        if !target.id.is_empty() {
            self.settings.write().unwrap().encodings.insert(target.id.clone(), encoding);
            self.save_settings();
        }
        self.end_run(app);
        self.engine.lock().unwrap().encoding = encoding;
        let _ = app.emit("target", target);
    }
}

/// Watch which app has focus and keep the target up to date.
pub fn watch_frontmost(app: AppHandle) {
    std::thread::Builder::new()
        .name("okkhor-frontmost".into())
        .spawn(move || {
            let own_pid_name = std::env::current_exe()
                .ok()
                .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().to_lowercase()))
                .unwrap_or_default();
            let own_id = app.config().identifier.to_lowercase();
            let mut last = String::new();
            loop {
                std::thread::sleep(std::time::Duration::from_millis(350));
                let Some(info) = okkhor_input::frontmost_app() else { continue };
                let id = info.id.to_lowercase();
                if id == last {
                    continue;
                }
                let own = id == own_id || id == own_pid_name || info.name == "BanglaOkkhor";
                last = id;
                if own {
                    // Typing moved into one of our windows: never edit across apps.
                    let st = app.state::<AppState>();
                    st.end_run(&app);
                    st.engine.lock().unwrap().encoding = Encoding::Unicode;
                    continue;
                }
                app.state::<AppState>().set_target(&app, info);
            }
        })
        .expect("spawn frontmost watcher");
}

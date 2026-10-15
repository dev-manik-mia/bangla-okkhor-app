//! SutonnyMJ: the legacy ANSI Bangla font. It is commercial, so we never ship
//! it; we use the copy installed on the system, or one the user imports.

use std::path::{Path, PathBuf};

use tauri::ipc::{InvokeBody, Request, Response};
use tauri::{AppHandle, Emitter, Manager};

const IMPORTED: &str = "SutonnyMJ.ttf";

fn font_dirs() -> Vec<PathBuf> {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).map(PathBuf::from);
    let mut dirs = Vec::new();
    if cfg!(target_os = "macos") {
        dirs.extend(home.iter().map(|h| h.join("Library/Fonts")));
        dirs.push("/Library/Fonts".into());
        dirs.push("/System/Library/Fonts".into());
    } else if cfg!(target_os = "windows") {
        let windir = std::env::var_os("WINDIR").map(PathBuf::from).unwrap_or_else(|| "C:\\Windows".into());
        dirs.push(windir.join("Fonts"));
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            dirs.push(PathBuf::from(local).join("Microsoft\\Windows\\Fonts"));
        }
    } else {
        dirs.extend(home.iter().flat_map(|h| [h.join(".local/share/fonts"), h.join(".fonts")]));
        dirs.push("/usr/share/fonts".into());
        dirs.push("/usr/local/share/fonts".into());
    }
    dirs
}

/// Prefer the regular face: "SutonnyMJ.ttf" over "SutonnyMJ Bold.ttf".
fn score(name: &str) -> Option<u8> {
    let n = name.to_lowercase();
    if !(n.contains("sutonny") && n.contains("mj")) || !(n.ends_with(".ttf") || n.ends_with(".otf")) {
        return None;
    }
    let styled = ["bold", "italic", "oblique"].iter().any(|s| n.contains(s));
    Some(if styled { 1 } else { 0 })
}

fn search(dir: &Path, depth: u8, best: &mut Option<(u8, PathBuf)>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let path = e.path();
        if path.is_dir() {
            if depth > 0 {
                search(&path, depth - 1, best);
            }
        } else if let Some(s) = score(&e.file_name().to_string_lossy()) {
            if best.as_ref().is_none_or(|(b, _)| s < *b) {
                *best = Some((s, path));
            }
        }
    }
}

fn imported_path(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_config_dir().ok().map(|d| d.join("fonts").join(IMPORTED))
}

/// The SutonnyMJ file to use: the imported copy first, then the system's.
pub fn find(app: &AppHandle) -> Option<PathBuf> {
    if let Some(p) = imported_path(app).filter(|p| p.is_file()) {
        return Some(p);
    }
    let mut best = None;
    for dir in font_dirs() {
        search(&dir, 3, &mut best);
    }
    best.map(|(_, p)| p)
}

/// Raw font bytes (empty when SutonnyMJ is not available).
#[tauri::command]
pub fn sutonny_font(app: AppHandle) -> Response {
    let bytes = find(&app).and_then(|p| std::fs::read(p).ok()).unwrap_or_default();
    Response::new(bytes)
}

/// Where the font came from: "imported", "system" or "missing".
#[tauri::command]
pub fn sutonny_status(app: AppHandle) -> &'static str {
    match find(&app) {
        Some(p) if Some(&p) == imported_path(&app).as_ref() => "imported",
        Some(_) => "system",
        None => "missing",
    }
}

fn is_font(bytes: &[u8]) -> bool {
    // TrueType, OpenType (CFF) and "true" (old Mac TrueType) signatures.
    matches!(bytes.get(..4), Some([0, 1, 0, 0]) | Some(b"OTTO") | Some(b"true"))
}

/// Save a user-supplied SutonnyMJ .ttf (sent as the raw request body).
#[tauri::command]
pub fn import_sutonny_font(app: AppHandle, request: Request<'_>) -> Result<(), String> {
    let InvokeBody::Raw(bytes) = request.body() else { return Err("expected the font file bytes".into()) };
    if !is_font(bytes) {
        return Err("This is not a TrueType or OpenType font file.".into());
    }
    let path = imported_path(&app).ok_or("no config directory")?;
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    let _ = app.emit("sutonny-font", ());
    Ok(())
}

#[tauri::command]
pub fn remove_sutonny_font(app: AppHandle) {
    if let Some(p) = imported_path(&app) {
        let _ = std::fs::remove_file(p);
    }
    let _ = app.emit("sutonny-font", ());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_regular_sutonny_files() {
        assert_eq!(score("SutonnyMJ.ttf"), Some(0));
        assert_eq!(score("SUTONNYMJ.TTF"), Some(0));
        assert_eq!(score("SutonnyMJ Bold.ttf"), Some(1));
        assert_eq!(score("SutonnyOMJ.ttf"), Some(0));
        assert_eq!(score("Sutonny.ttf"), None);
        assert_eq!(score("SutonnyMJ.txt"), None);
    }

    #[test]
    fn finds_sutonny_in_nested_font_folders() {
        let dir = std::env::temp_dir().join(format!("okkhor-fonts-{}", std::process::id()));
        let nested = dir.join("truetype/bijoy");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("SutonnyMJ Bold.ttf"), b"x").unwrap();
        std::fs::write(nested.join("SutonnyMJ.ttf"), b"x").unwrap();
        std::fs::write(dir.join("NotoSansBengali.ttf"), b"x").unwrap();
        let mut best = None;
        search(&dir, 3, &mut best);
        assert_eq!(best.map(|(_, p)| p), Some(nested.join("SutonnyMJ.ttf")));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn checks_font_signatures() {
        assert!(is_font(&[0, 1, 0, 0, 9]));
        assert!(is_font(b"OTTO...."));
        assert!(!is_font(b"PK\x03\x04"));
    }
}

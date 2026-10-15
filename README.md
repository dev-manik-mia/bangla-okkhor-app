# BanglaOkkhor — desktop app

Bijoy-style Bangla keyboard for **macOS, Windows and Linux**, built with Tauri 2 + Rust.
It types Bangla system-wide (Word, Google Docs in any browser, Pages, Notes, Notepad, LibreOffice…),
and ships an on-screen keyboard, word suggestions, a floating widget, a mode indicator and a
Unicode ↔ SutonnyMJ ANSI converter.

## Layout

```
app/
  crates/okkhor-core/    layout, typing engine, Unicode↔ANSI converter, dictionary (no UI, unit-tested)
  crates/okkhor-input/   system-wide keyboard hook + text injection per OS
  src-tauri/             the Tauri app: tray, windows, settings, commands
  src/                   React UI (Preferences, Keyboard, Converter, Onboarding, Widget, Indicator)
```

## Develop

Requirements: Rust (stable), Node 20+, and on Linux the Tauri system packages
(`libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`).

```sh
npm install
npm run tauri dev        # run the app
cargo test -p okkhor-core  # engine + converter tests
npm run tauri build      # installers for the current OS (.dmg/.app, .msi/.exe, .deb/.rpm/.AppImage)
```

`.github/workflows/build.yml` builds all three platforms on GitHub Actions (macOS universal binary).

## How typing works

Instead of a per-OS input-method framework, BanglaOkkhor installs a low-level keyboard hook and
types through the OS:

| OS | Capture | Output | Permission |
|---|---|---|---|
| macOS 11+ | Quartz event tap | Unicode `CGEvent`s | Accessibility (asked during setup) |
| Windows 10+ | `WH_KEYBOARD_LL` / `WH_MOUSE_LL` | `SendInput` + `KEYEVENTF_UNICODE` | none |
| Linux | evdev grab, unhandled keys re-emitted via uinput | XTest (X11 / XWayland) | `input` group + uinput udev rule |

Keys are mapped by **physical position** (scan code / key code), so the layout works over any
Latin layout. The engine keeps only the text it inserted in the current run and answers each key
with "delete N characters, insert this". It never edits earlier text. The run ends on space,
Enter, Backspace, arrows, a click, a ⌘/Ctrl/Alt shortcut or an app switch.

In ANSI mode (the widget shows the switch for Word, Pages, LibreOffice and WPS), the same run is
rendered through the SutonnyMJ converter, so legacy documents receive glyphs in visual order.
The choice is remembered per app.

### Notes per platform

- **macOS**: when running from `tauri dev`, the Accessibility permission belongs to the
  terminal/IDE that launched it. A bundled `BanglaOkkhor.app` asks under its own name. Floating
  windows (keyboard, widget, indicator) are converted to non-activating `NSPanel`s, so clicking
  them never takes focus from the app you are typing in. Password fields (Secure Input) are
  never seen by the hook.
- **Linux**: run `sudo usermod -aG input $USER`, install the bundled udev rule
  (`src-tauri/linux/70-banglaokkhor.rules`; the .deb and .rpm install it) and log in again.
  Typing works in X11 and XWayland apps. Native GNOME/KDE Wayland apps cannot receive synthetic
  text from outside the compositor.
- **Windows**: Ctrl+Alt is AltGr on some layouts. If a shortcut clashes, record a different one
  in Preferences → Shortcuts.

## Default shortcuts

| Action | Shortcut |
|---|---|
| Switch Bangla / English | ⌃⌥B (Ctrl+Alt+B) |
| On-screen keyboard | ⌃⌥K |
| Converter | ⌃⌥C |
| Floating widget | ⌃⌥W |
| Insert ৳ | ⌃⌥4 |

While the widget shows suggestions, **1–4** pick one.

## Branding

The product is **BanglaOkkhor** (বাংলাঅক্ষর). "Bijoy-style" describes the key layout only; no
Bijoy names, logos or assets are used. Bundled fonts are SIL OFL (via Fontsource).

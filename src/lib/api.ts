import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

export type Encoding = "unicode" | "ansi";
export type Lang = "en" | "bn";
export type Layout = "bijoy" | "avro";

export interface Shortcut {
  ctrl: boolean;
  alt: boolean;
  shift: boolean;
  meta: boolean;
  code: string;
}

export interface Shortcuts {
  toggle: Shortcut;
  keyboard: Shortcut;
  converter: Shortcut;
  widget: Shortcut;
  taka: Shortcut;
}

export interface Settings {
  launch: boolean;
  indicator: boolean;
  sugg: boolean;
  sound: boolean;
  bnDigits: boolean;
  reorder: boolean;
  autoVowel: boolean;
  reph: boolean;
  layout: Layout;
  startIn: "bn" | "en";
  theme: "system" | "light" | "dark";
  lang: Lang;
  font: string;
  fontSize: number;
  shortcuts: Shortcuts;
  encodings: Record<string, Encoding>;
  onboarded: boolean;
}

export interface Target {
  name: string;
  id: string;
  wordProcessor: boolean;
  encoding: Encoding;
}

export interface PermissionInfo {
  granted: boolean;
  message: string | null;
  running: boolean;
}

export interface LayoutKey {
  code: string;
  bn: string;
  bnShift: string;
  lat: string;
  latShift: string;
}

export type Platform = "macos" | "windows" | "linux";

export interface Bootstrap {
  settings: Settings;
  bangla: boolean;
  target: Target;
  suggestions: string[];
  permission: PermissionInfo;
  platform: Platform;
  layout: LayoutKey[];
  version: string;
}

export const api = {
  bootstrap: () => invoke<Bootstrap>("bootstrap"),
  setSettings: (patch: Partial<Settings> | Record<string, unknown>) => invoke<Settings>("set_settings", { patch }),
  saveSettings: () => invoke<void>("save_settings"),
  setMode: (bangla: boolean) => invoke<void>("set_mode", { bangla }),
  toggleMode: () => invoke<boolean>("toggle_mode"),
  setInput: (input: Layout | "en") => invoke<void>("set_input", { input }),
  virtualKey: (code: string, shift: boolean) => invoke<void>("virtual_key", { code, shift }),
  acceptSuggestion: (word: string) => invoke<void>("accept_suggestion", { word }),
  convert: (text: string, to: "ansi" | "unicode") => invoke<string>("convert", { text, to }),
  setEncoding: (encoding: Encoding) => invoke<void>("set_encoding", { encoding }),
  openWindow: (name: string) => invoke<void>("open_window", { name }),
  hideWindow: (name: string) => invoke<void>("hide_window", { name }),
  setRecording: (on: boolean) => invoke<void>("set_recording", { on }),
  setKeyboardFocus: (on: boolean) => invoke<void>("set_keyboard_focus", { on }),
  permissionStatus: () => invoke<PermissionInfo>("permission_status"),
  requestPermission: () => invoke<PermissionInfo>("request_permission"),
  finishOnboarding: () => invoke<void>("finish_onboarding"),
  convertSelection: () => invoke<boolean>("convert_selection"),
  quit: () => invoke<void>("quit"),
};

/** App-wide state shared by every window, kept live through backend events. */
export function useApp(): Bootstrap | null {
  const [boot, setBoot] = useState<Bootstrap | null>(null);
  useEffect(() => {
    let alive = true;
    const un: Array<() => void> = [];
    api.bootstrap().then((b) => alive && setBoot(b));
    const on = <T,>(name: string, f: (b: Bootstrap, v: T) => Bootstrap) =>
      listen<T>(name, (e) => setBoot((b) => (b ? f(b, e.payload) : b))).then((u) => un.push(u));
    on<Settings>("settings", (b, settings) => ({ ...b, settings }));
    on<boolean>("mode", (b, bangla) => ({ ...b, bangla }));
    on<Target>("target", (b, target) => ({ ...b, target }));
    on<string[]>("suggestions", (b, suggestions) => ({ ...b, suggestions }));
    on<PermissionInfo>("permission", (b, permission) => ({ ...b, permission }));
    return () => {
      alive = false;
      un.forEach((u) => u());
    };
  }, []);
  return boot;
}

const KEY_NAMES: Record<string, string> = { Space: "Space", Backquote: "`", Minus: "-", Equal: "=", BracketLeft: "[", BracketRight: "]", Backslash: "\\", Semicolon: ";", Quote: "'", Comma: ",", Period: ".", Slash: "/" };

/** "⌃⌥B" on macOS, "Ctrl+Alt+B" elsewhere. */
export function shortcutLabel(s: Shortcut, platform: Platform): string {
  if (!s.code) return "—";
  const key = s.code.startsWith("Key") ? s.code.slice(3) : s.code.startsWith("Digit") ? s.code.slice(5) : KEY_NAMES[s.code] ?? s.code;
  if (platform === "macos") return (s.ctrl ? "⌃" : "") + (s.alt ? "⌥" : "") + (s.shift ? "⇧" : "") + (s.meta ? "⌘" : "") + key;
  return [s.ctrl && "Ctrl", s.alt && "Alt", s.shift && "Shift", s.meta && "Win", key].filter(Boolean).join("+");
}

import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect } from "react";
import type { Bootstrap } from "./api";

const media = window.matchMedia("(prefers-color-scheme: dark)");

/** Apply light/dark, platform and glass classes to <html>, and the Bangla font tokens. */
export function useTheme(boot: Bootstrap | null) {
  const pref = boot?.settings.theme ?? "system";
  const platform = boot?.platform;
  const font = boot?.settings.font ?? "Noto Sans Bengali";
  const size = boot?.settings.fontSize ?? 20;
  const lang = boot?.settings.lang ?? "en";

  useEffect(() => {
    const root = document.documentElement;
    const apply = () => {
      const dark = pref === "dark" || (pref === "system" && media.matches);
      root.dataset.theme = dark ? "dark" : "light";
    };
    apply();
    media.addEventListener("change", apply);
    getCurrentWindow()
      .setTheme(pref === "system" ? null : pref)
      .catch(() => {});
    return () => media.removeEventListener("change", apply);
  }, [pref]);

  useEffect(() => {
    if (!platform) return;
    const root = document.documentElement;
    root.dataset.platform = platform;
    // macOS and Windows windows sit on native vibrancy / Mica.
    root.classList.toggle("glass", platform !== "linux");
  }, [platform]);

  useEffect(() => {
    const root = document.documentElement;
    root.style.setProperty("--editor-font", `'${font}', 'Noto Sans Bengali', sans-serif`);
    root.style.setProperty("--editor-size", `${size}px`);
    root.lang = lang;
  }, [font, size, lang]);
}

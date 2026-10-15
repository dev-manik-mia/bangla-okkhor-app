import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

/** CSS family name we register the SutonnyMJ file under. */
export const SUTONNY_FAMILY = "OkkhorSutonnyMJ";
export type SutonnyStatus = "system" | "imported" | "missing" | "loading";

let loaded: FontFace | null = null;

async function load(): Promise<SutonnyStatus> {
  const status = await invoke<SutonnyStatus>("sutonny_status");
  if (loaded) {
    document.fonts.delete(loaded);
    loaded = null;
  }
  if (status === "missing") return status;
  const bytes = await invoke<ArrayBuffer>("sutonny_font");
  if (!bytes.byteLength) return "missing";
  const face = new FontFace(SUTONNY_FAMILY, bytes);
  await face.load();
  document.fonts.add(face);
  loaded = face;
  return status;
}

/** Loads SutonnyMJ (system or imported) into this window and tracks changes. */
export function useSutonny(): SutonnyStatus {
  const [status, setStatus] = useState<SutonnyStatus>("loading");
  useEffect(() => {
    const refresh = () => load().then(setStatus, () => setStatus("missing"));
    refresh();
    const un = listen("sutonny-font", refresh);
    return () => {
      un.then((u) => u());
    };
  }, []);
  return status;
}

export async function importSutonny(file: File): Promise<void> {
  const buf = await file.arrayBuffer();
  await invoke("import_sutonny_font", new Uint8Array(buf));
}

export const removeSutonny = () => invoke<void>("remove_sutonny_font");

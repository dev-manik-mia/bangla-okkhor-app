import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";
import { LayoutPicker } from "../components/LayoutPicker";
import { Segmented, ToggleRow } from "../components/ui";
import { api, shortcutLabel, type Bootstrap, type Lang, type Settings, type Shortcut, type Shortcuts } from "../lib/api";
import { importSutonny, removeSutonny, SUTONNY_FAMILY, useSutonny } from "../lib/fonts";
import { playKeySound, SOUND_KINDS, type SoundKind } from "../lib/sounds";
import { t } from "../lib/i18n";

type Tab = "general" | "layouts" | "shortcuts" | "fonts";
const TABS: [Tab, string][] = [
  ["general", "স"],
  ["layouts", "অ"],
  ["shortcuts", "⌘"],
  ["fonts", "ফ"],
];
const FONTS = ["Noto Sans Bengali", "Hind Siliguri", "Baloo Da 2", "Tiro Bangla", "Anek Bangla", "Galada"];
const DEFAULT_SC: Shortcuts = {
  toggle: { ctrl: true, alt: true, shift: false, meta: false, code: "KeyB" },
  layout: { ctrl: true, alt: true, shift: false, meta: false, code: "KeyL" },
  bijoy: { ctrl: true, alt: true, shift: false, meta: false, code: "Digit1" },
  avro: { ctrl: true, alt: true, shift: false, meta: false, code: "Digit2" },
  english: { ctrl: true, alt: true, shift: false, meta: false, code: "Digit3" },
  keyboard: { ctrl: true, alt: true, shift: false, meta: false, code: "KeyK" },
  converter: { ctrl: true, alt: true, shift: false, meta: false, code: "KeyC" },
  widget: { ctrl: true, alt: true, shift: false, meta: false, code: "KeyW" },
  taka: { ctrl: true, alt: true, shift: false, meta: false, code: "Digit4" },
};
const RECORDABLE = /^(Key[A-Z]|Digit\d|Backquote|Minus|Equal|BracketLeft|BracketRight|Backslash|Semicolon|Quote|Comma|Period|Slash|Space)$/;

/** Plays the chosen click for every key while "Key click sound" is on. */
function useKeySound(on: boolean, kind: SoundKind) {
  useEffect(() => {
    if (!on) return;
    const un = listen("key", () => playKeySound(kind));
    return () => {
      un.then((u) => u());
    };
  }, [on, kind]);
}

export function Preferences({ boot }: { boot: Bootstrap }) {
  const st = boot.settings;
  const s = t(st.lang);
  const [tab, setTab] = useState<Tab>("general");
  useKeySound(st.sound, st.soundKind);

  useEffect(() => {
    const un = listen<string>("prefs-tab", (e) => setTab(e.payload as Tab));
    return () => {
      un.then((u) => u());
    };
  }, []);

  const set = (patch: Partial<Settings> | Record<string, unknown>) => api.setSettings(patch);
  const tog = (key: keyof Settings & keyof typeof s) => (
    <ToggleRow key={key} label={s[key] as string} on={st[key] as boolean} onChange={(v) => set({ [key]: v })} />
  );

  return (
    <div className="prefs">
      <nav className="prefs-side" data-tauri-drag-region>
        {TABS.map(([id, icon]) => (
          <button key={id} className={"prefs-tab" + (tab === id ? " on" : "")} onClick={() => setTab(id)}>
            <span className="prefs-icon bn">{icon}</span>
            <span>{s[id]}</span>
          </button>
        ))}
        <span style={{ flex: 1 }} data-tauri-drag-region />
        <span className="sub" style={{ fontSize: 11, padding: "0 8px" }}>
          {s.appName} {boot.version}
        </span>
      </nav>
      <main className="prefs-main">
        <h1 data-tauri-drag-region>{s[tab]}</h1>

        {tab === "general" && (
          <>
            <div className="card">
              {(["launch", "sugg", "sound"] as const).map(tog)}
              {st.sound && (
                <div className="row">
                  <span>{s.soundKind}</span>
                  <Segmented<SoundKind>
                    value={st.soundKind}
                    options={SOUND_KINDS.map((k) => [k, s.sounds[k]])}
                    onChange={(v) => {
                      playKeySound(v);
                      set({ soundKind: v });
                    }}
                  />
                </div>
              )}
              {tog("bnDigits")}
            </div>
            <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
              <span className="sub" style={{ fontSize: 12, fontWeight: 600, padding: "0 4px" }}>
                {s.floating}
              </span>
              <div className="card">{(["indicator", "widget"] as const).map(tog)}</div>
              <span className="sub" style={{ fontSize: 12, padding: "0 4px" }}>
                {s.floatingHint}
              </span>
            </div>
            <div className="card">
              <div className="row">
                <span>{s.startIn}</span>
                <Segmented value={st.startIn} options={[["bn", s.bn], ["en", s.en]]} onChange={(v) => set({ startIn: v })} />
              </div>
              <div className="row">
                <span>{s.appearance}</span>
                <Segmented
                  value={st.theme}
                  options={[["system", s.system], ["light", s.light], ["dark", s.dark]]}
                  onChange={(v) => set({ theme: v })}
                />
              </div>
              <div className="row">
                <span>{s.interfaceLang}</span>
                <Segmented value={st.lang} options={[["en", "English"], ["bn", <span className="bn">বাংলা</span>]]} onChange={(v) => set({ lang: v })} />
              </div>
            </div>
          </>
        )}

        {tab === "layouts" && (
          <>
            <LayoutPicker
              boot={boot}
              extra={(id) =>
                id === "bijoy" && (
                  <button
                    className="btn"
                    onClick={(e) => {
                      e.stopPropagation();
                      api.openWindow("keyboard");
                    }}
                  >
                    {s.viewLayout}
                  </button>
                )
              }
            />
            <div className="tiles bn">
              {(st.layout === "avro"
                ? [
                    ["ami", "আমি"],
                    ["bangla", "বাংলা"],
                    ["kemon acho", "কেমন আছ"],
                  ]
                : [
                    ["d + j", "কি"],
                    ["c + j + f", "কো"],
                    ["g + f", "আ"],
                  ]
              ).map(([k, v]) => (
                <div className="card tile" key={k}>
                  <span className="mono sub" style={{ fontSize: 12 }}>
                    {k}
                  </span>
                  <span style={{ fontSize: 22 }}>{v}</span>
                </div>
              ))}
            </div>
            {st.layout === "bijoy" && <div className="card">{(["reorder", "autoVowel", "reph"] as const).map(tog)}</div>}
          </>
        )}

        {tab === "shortcuts" && <ShortcutsTab boot={boot} />}

        {tab === "fonts" && (
          <>
            <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
              <span className="sub">{s.defaultFont}</span>
              <label style={{ display: "flex", alignItems: "center", gap: 10 }}>
                <span className="sub">{s.size}</span>
                <input
                  type="range"
                  min={12}
                  max={32}
                  value={st.fontSize}
                  onChange={(e) => set({ fontSize: +e.target.value })}
                  style={{ width: 120, accentColor: "var(--acc)" }}
                />
                <span className="mono" style={{ width: 44, fontSize: 12 }}>
                  {st.fontSize} pt
                </span>
              </label>
            </div>
            <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
              {FONTS.map((f) => (
                <button key={f} className={"font-row" + (st.font === f ? " on" : "")} onClick={() => set({ font: f })}>
                  <span style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column", gap: 2, textAlign: "left" }}>
                    <span className="sub" style={{ fontSize: 11 }}>
                      {f}
                    </span>
                    <span className="ellipsis" style={{ fontSize: st.fontSize * 0.9, fontFamily: `'${f}', sans-serif` }}>
                      {s.sample}
                    </span>
                  </span>
                  <span style={{ color: "var(--acc)", fontWeight: 700 }}>{st.font === f ? "✓" : ""}</span>
                </button>
              ))}
            </div>
            <SutonnySection lang={st.lang} />
          </>
        )}
      </main>
    </div>
  );
}

function ShortcutsTab({ boot }: { boot: Bootstrap }) {
  const st = boot.settings;
  const s = t(st.lang);
  const [rec, setRec] = useState<keyof Shortcuts | null>(null);
  const [note, setNote] = useState("");
  const rows: [keyof Shortcuts, string][] = [
    ["toggle", s.toggleLang],
    ["layout", s.switchLayout],
    ["bijoy", s.toBijoy],
    ["avro", s.toAvro],
    ["english", s.toEnglish],
    ["keyboard", s.openKb],
    ["converter", s.openConv],
    ["widget", s.openWidget],
    ["taka", s.taka],
  ];

  useEffect(() => {
    api.setRecording(rec !== null);
    if (!rec) return;
    const kd = (e: KeyboardEvent) => {
      e.preventDefault();
      if (e.key === "Escape") return setRec(null);
      if (["Shift", "Control", "Alt", "Meta"].includes(e.key)) return;
      if (!RECORDABLE.test(e.code) || !(e.ctrlKey || e.altKey || e.metaKey)) return;
      const sc: Shortcut = { ctrl: e.ctrlKey, alt: e.altKey, shift: e.shiftKey, meta: e.metaKey, code: e.code };
      // One key combination, one action: clear any other action that had it.
      const taken = rows.find(([id]) => id !== rec && sameShortcut(st.shortcuts[id], sc));
      const patch: Record<string, Partial<Shortcut>> = { [rec]: sc };
      if (taken) patch[taken[0]] = { code: "" };
      setNote(taken ? s.scTaken.replace("{shortcut}", shortcutLabel(sc, boot.platform)).replace("{action}", taken[1]) : "");
      api.setSettings({ shortcuts: patch });
      setRec(null);
    };
    // Leaving the window stops recording, so shortcuts never stay switched off.
    const blur = () => setRec(null);
    window.addEventListener("keydown", kd, true);
    window.addEventListener("blur", blur);
    return () => {
      window.removeEventListener("keydown", kd, true);
      window.removeEventListener("blur", blur);
    };
  }, [rec]);
  useEffect(() => () => void api.setRecording(false), []);

  return (
    <>
      <div className="sub">{s.scHint}</div>
      {note && (
        <div className="sub" role="status" style={{ color: "var(--acc-2)" }}>
          {note}
        </div>
      )}
      <div className="card">
        {rows.map(([id, label]) => (
          <div className="row" key={id}>
            <span>{label}</span>
            <button className={"sc-key mono" + (rec === id ? " rec" : "")} onClick={() => setRec(rec === id ? null : id)}>
              {rec === id ? s.record : shortcutLabel(st.shortcuts[id], boot.platform)}
            </button>
          </div>
        ))}
      </div>
      <div style={{ display: "flex", justifyContent: "flex-end" }}>
        <button
          className="btn"
          onClick={() => {
            setRec(null);
            setNote("");
            api.setSettings({ shortcuts: DEFAULT_SC });
          }}
        >
          {s.reset}
        </button>
      </div>
    </>
  );
}

function sameShortcut(a: Shortcut, b: Shortcut) {
  return a.code === b.code && a.ctrl === b.ctrl && a.alt === b.alt && a.shift === b.shift && a.meta === b.meta;
}

function SutonnySection({ lang }: { lang: Lang }) {
  const s = t(lang);
  const status = useSutonny();
  const [error, setError] = useState("");
  const input = useRef<HTMLInputElement>(null);
  const label = { system: s.fontSystem, imported: s.fontImported, missing: s.fontMissing, loading: "…" }[status];
  const ok = status === "system" || status === "imported";
  return (
    <div className="card" style={{ padding: 14, display: "flex", flexDirection: "column", gap: 10 }}>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 12 }}>
        <span style={{ fontWeight: 600 }}>{s.ansiFont}</span>
        <span className="sub" style={{ fontSize: 12, color: ok ? "var(--acc)" : undefined }}>
          {ok ? "✓ " : ""}
          {label}
        </span>
      </div>
      <div className="sub" style={{ fontSize: 12 }}>
        {s.ansiFontDesc}
      </div>
      {ok && (
        <div className="ellipsis" style={{ fontSize: 22, fontFamily: `'${SUTONNY_FAMILY}'` }}>
          Avgvi †mvbvi evsjv, Avwg †Zvgvq fv‡jvevwm|
        </div>
      )}
      <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
        <button className="btn" onClick={() => input.current?.click()}>
          {s.addFont}
        </button>
        {status === "imported" && (
          <button className="btn" onClick={() => removeSutonny()}>
            {s.removeFont}
          </button>
        )}
        {error && (
          <span style={{ fontSize: 12, color: "var(--acc-2)" }} role="alert">
            {error}
          </span>
        )}
        <input
          ref={input}
          type="file"
          accept=".ttf,.otf,font/ttf,font/otf"
          hidden
          onChange={(e) => {
            const file = e.target.files?.[0];
            e.target.value = "";
            if (!file) return;
            setError("");
            importSutonny(file).catch((err) => setError(String(err)));
          }}
        />
      </div>
    </div>
  );
}

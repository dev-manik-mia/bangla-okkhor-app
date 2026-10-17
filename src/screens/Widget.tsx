import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { useEffect, useRef, useState } from "react";
import { ModeDot, Segmented } from "../components/ui";
import { api, shortcutLabel, type Bootstrap, type Encoding, type Layout } from "../lib/api";
import { useDrag } from "../lib/drag";
import { t } from "../lib/i18n";

const WIDTH = 300;

export function Widget({ boot }: { boot: Bootstrap }) {
  const s = t(boot.settings.lang);
  const [mini, setMini] = useState(false);
  const [note, setNote] = useState("");
  const win = getCurrentWindow();
  const drag = useDrag(win);

  const typingOff = !boot.permission.running;
  const content = useRef<HTMLDivElement>(null);

  // The window follows the content's height (warning, switch, encoding row…).
  useEffect(() => {
    const el = content.current;
    if (!el) return;
    const fit = () => win.setSize(new LogicalSize(WIDTH, Math.ceil(el.getBoundingClientRect().height) + 2)).catch(() => {});
    const ro = new ResizeObserver(fit);
    ro.observe(el);
    fit();
    return () => ro.disconnect();
  }, []);

  const input: Layout | "en" = boot.bangla ? boot.settings.layout : "en";
  const sc = boot.settings.shortcuts;
  const switchHint = [
    `${s.toggleLang}: ${shortcutLabel(sc.toggle, boot.platform)}`,
    `${s.inputBijoy}: ${shortcutLabel(sc.bijoy, boot.platform)}`,
    `${s.inputAvro}: ${shortcutLabel(sc.avro, boot.platform)}`,
    `${s.inputEn}: ${shortcutLabel(sc.english, boot.platform)}`,
  ].join(" · ");

  const convert = async () => {
    const ok = await api.convertSelection().catch(() => false);
    if (ok) {
      setNote(s.w.converted);
      setTimeout(() => setNote(""), 1500);
    } else {
      await api.openWindow("converter");
    }
  };

  const target = boot.target;
  return (
    <div className="widget">
      <div ref={content}>
      <div
        className="widget-head"
        title={s.w.hint}
        onPointerDown={drag.down}
        onPointerMove={drag.move}
        onPointerUp={drag.up}
        onPointerCancel={drag.up}
      >
        <button className="mode-pill" onClick={() => api.toggleMode()} title={switchHint}>
          <ModeDot bangla={boot.bangla} />
          <span className="bn" style={{ fontWeight: 600, fontSize: 12 }}>
            {boot.bangla ? (boot.settings.layout === "avro" ? "অভ্র" : "বিজয়") : "English"}
          </span>
        </button>
        <span className="sub ellipsis" style={{ flex: 1, fontSize: 11 }}>
          {note || (target.name ? `${s.w.typingIn} · ${target.name}` : s.w.nothing)}
        </span>
        <span className="grip" aria-hidden>
          <i />
          <i />
          <i />
          <i />
        </span>
        <button className="icon-btn" aria-label="Minimise" onClick={() => setMini(!mini)}>
          {mini ? "▢" : "–"}
        </button>
        <button className="icon-btn" aria-label={s.hideTyping} title={s.hideTyping} onClick={() => api.setSettings({ widget: false })}>
          ×
        </button>
      </div>
      {!mini && (
        <div className="widget-body">
          {typingOff && (
            <div className="widget-warn" role="alert">
              <span>⚠︎ {s.typingOff}</span>
              <button className="btn sm primary" onClick={() => api.fixAccess()}>
                {s.fixAccess}
              </button>
            </div>
          )}
          <div title={switchHint}>
            <Segmented<Layout | "en">
              fill
              value={input}
              options={[
                ["bijoy", <span className="bn">{s.inputBijoy}</span>],
                ["avro", <span className="bn">{s.inputAvro}</span>],
                ["en", s.inputEn],
              ]}
              onChange={(v) => api.setInput(v)}
            />
          </div>
          <div className="widget-sugg">
            {boot.suggestions.length > 0 ? (
              boot.suggestions.map((w, i) => (
                <button key={w} className="sugg" style={{ fontSize: 14 }} onClick={() => api.acceptSuggestion(w)}>
                  <span className="n">{i + 1}</span>
                  {w}
                </button>
              ))
            ) : (
              <span className="sub" style={{ padding: "0 6px", fontSize: 11 }}>
                {s.suggestions}
              </span>
            )}
          </div>
          <div style={{ display: "flex", gap: 6 }}>
            <button className="btn sm" style={{ flex: 1 }} onClick={() => api.openWindow("keyboard")}>
              ⌨ {s.w.kb}
            </button>
            <button className="btn sm" style={{ flex: 1 }} onClick={convert}>
              ⇄ {s.w.conv}
            </button>
          </div>
          {target.wordProcessor && (
            <Segmented<Encoding>
              fill
              value={target.encoding}
              options={[["unicode", s.w.uni], ["ansi", s.w.ansi]]}
              onChange={(v) => api.setEncoding(v)}
            />
          )}
        </div>
      )}
      </div>
    </div>
  );
}


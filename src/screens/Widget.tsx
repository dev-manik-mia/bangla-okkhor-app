import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { useEffect, useState } from "react";
import { ModeDot, Segmented } from "../components/ui";
import { api, type Bootstrap, type Encoding } from "../lib/api";
import { t } from "../lib/i18n";

const FULL = new LogicalSize(300, 156);
const MINI = new LogicalSize(300, 46);

export function Widget({ boot }: { boot: Bootstrap }) {
  const s = t(boot.settings.lang);
  const [mini, setMini] = useState(false);
  const [note, setNote] = useState("");
  const win = getCurrentWindow();

  useEffect(() => {
    win.setSize(mini ? MINI : FULL).catch(() => {});
  }, [mini]);

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
      <div
        className="widget-head"
        title={s.w.hint}
        onPointerDown={(e) => {
          if ((e.target as HTMLElement).closest("button")) return;
          win.startDragging();
        }}
      >
        <button className="mode-pill" onClick={() => api.toggleMode()}>
          <ModeDot bangla={boot.bangla} />
          <span className="bn" style={{ fontWeight: 600, fontSize: 12 }}>
            {boot.bangla ? "বাংলা" : "English"}
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
      </div>
      {!mini && (
        <div className="widget-body">
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
  );
}


import { getCurrentWindow, LogicalSize, PhysicalPosition } from "@tauri-apps/api/window";
import { useEffect, useRef, useState, type PointerEvent } from "react";
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
  const drag = useDrag(win);

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
        onPointerDown={drag.down}
        onPointerMove={drag.move}
        onPointerUp={drag.up}
        onPointerCancel={drag.up}
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


/**
 * Moves the window by following the pointer. The widget is a non-activating
 * panel, where the OS-level drag (`startDragging`) is ignored on macOS, so we
 * position the window ourselves. Pointer capture keeps the moves coming even
 * when the cursor runs ahead of the window.
 */
function useDrag(win: ReturnType<typeof getCurrentWindow>) {
  const start = useRef<{ x: number; y: number; sx: number; sy: number; scale: number } | null>(null);
  const pending = useRef<{ x: number; y: number } | null>(null);
  const frame = useRef(0);

  const down = async (e: PointerEvent<HTMLDivElement>) => {
    if (e.button !== 0 || (e.target as HTMLElement).closest("button")) return;
    e.currentTarget.setPointerCapture(e.pointerId);
    const { screenX: sx, screenY: sy } = e;
    const [pos, scale] = await Promise.all([win.outerPosition(), win.scaleFactor()]);
    start.current = { x: pos.x, y: pos.y, sx, sy, scale };
  };

  const move = (e: PointerEvent<HTMLDivElement>) => {
    const s = start.current;
    if (!s) return;
    pending.current = { x: Math.round(s.x + (e.screenX - s.sx) * s.scale), y: Math.round(s.y + (e.screenY - s.sy) * s.scale) };
    if (frame.current) return;
    frame.current = requestAnimationFrame(() => {
      frame.current = 0;
      const p = pending.current;
      if (p) win.setPosition(new PhysicalPosition(p.x, p.y)).catch(() => {});
    });
  };

  const up = (e: PointerEvent<HTMLDivElement>) => {
    if (!start.current) return;
    start.current = null;
    if (e.currentTarget.hasPointerCapture(e.pointerId)) e.currentTarget.releasePointerCapture(e.pointerId);
    api.saveSettings();
  };

  return { down, move, up };
}

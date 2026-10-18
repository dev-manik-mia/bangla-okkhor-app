import { getCurrentWindow } from "@tauri-apps/api/window";
import { ModeDot } from "../components/ui";
import { api, shortcutLabel, type Bootstrap, type Layout } from "../lib/api";
import { useDrag } from "../lib/drag";
import { t } from "../lib/i18n";

/** Always-visible pill: shows the active input and switches it with one click. Drag it by the dot or the grip. */
export function Indicator({ boot }: { boot: Bootstrap }) {
  const s = t(boot.settings.lang);
  const drag = useDrag(getCurrentWindow());
  const sc = boot.settings.shortcuts;
  const current: Layout | "en" = boot.bangla ? boot.settings.layout : "en";

  const grip = (
    <span className="grip ind-grip" title={s.dragPill} aria-hidden>
      <i />
      <i />
      <i />
      <i />
      <i />
      <i />
    </span>
  );
  const dragProps = { onPointerDown: drag.down, onPointerMove: drag.move, onPointerUp: drag.up, onPointerCancel: drag.up };

  if (!boot.permission.running) {
    return (
      <div className="indicator warn" {...dragProps}>
        {grip}
        <button className="ind-fix" onClick={() => api.fixAccess()} title={s.typingOff}>
          <ModeDot bangla={false} size={24} />
          <span className="ellipsis" style={{ fontSize: 12 }}>
            ⚠︎ {s.fixAccess}
          </span>
        </button>
      </div>
    );
  }

  const options: [Layout | "en", string, string][] = [
    ["bijoy", s.inputBijoy, shortcutLabel(sc.bijoy, boot.platform)],
    ["avro", s.inputAvro, shortcutLabel(sc.avro, boot.platform)],
    ["en", "EN", shortcutLabel(sc.english, boot.platform)],
  ];
  return (
    <div
      className="indicator"
      role="radiogroup"
      aria-label={s.chooseLayout}
      {...dragProps}
    >
      {grip}
      <ModeDot bangla={boot.bangla} size={24} />
      {options.map(([id, label, hint]) => (
        <button
          key={id}
          role="radio"
          aria-checked={current === id}
          className={"ind-opt" + (current === id ? " on" : "") + (id !== "en" ? " bn" : "")}
          title={`${label} · ${hint}`}
          onClick={() => api.setInput(id)}
        >
          {label}
        </button>
      ))}
      <button className="ind-close" aria-label={s.hideTyping} title={s.hideTyping} onClick={() => api.setSettings({ indicator: false })}>
        ×
      </button>
    </div>
  );
}

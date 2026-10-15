import { ModeDot } from "../components/ui";
import { api, shortcutLabel, type Bootstrap, type Layout } from "../lib/api";
import { t } from "../lib/i18n";

/** Always-visible pill: shows the active input and switches it with one click. */
export function Indicator({ boot }: { boot: Bootstrap }) {
  const s = t(boot.settings.lang);
  const sc = boot.settings.shortcuts;
  const current: Layout | "en" = boot.bangla ? boot.settings.layout : "en";

  if (!boot.permission.running) {
    return (
      <button className="indicator warn" onClick={() => api.fixAccess()} title={s.typingOff}>
        <ModeDot bangla={false} size={24} />
        <span className="ellipsis" style={{ fontSize: 12 }}>
          ⚠︎ {s.fixAccess}
        </span>
      </button>
    );
  }

  const options: [Layout | "en", string, string][] = [
    ["bijoy", s.inputBijoy, `${s.toggleLang}: ${shortcutLabel(sc.toggle, boot.platform)}`],
    ["avro", s.inputAvro, `${s.switchLayout}: ${shortcutLabel(sc.layout, boot.platform)}`],
    ["en", "EN", `${s.toggleLang}: ${shortcutLabel(sc.toggle, boot.platform)}`],
  ];
  return (
    <div className="indicator" role="radiogroup" aria-label={s.chooseLayout}>
      <ModeDot bangla={boot.bangla} size={24} />
      {options.map(([id, label, hint]) => (
        <button
          key={id}
          role="radio"
          aria-checked={current === id}
          className={"ind-opt" + (current === id ? " on" : "") + (id !== "en" ? " bn" : "")}
          title={hint}
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

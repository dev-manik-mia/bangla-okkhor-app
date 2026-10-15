import { api, type Bootstrap, type Layout } from "../lib/api";
import { t } from "../lib/i18n";

/** Pick Bijoy71 or Avro Phonetic. */
export function LayoutPicker({ boot, extra }: { boot: Bootstrap; extra?: (layout: Layout) => React.ReactNode }) {
  const s = t(boot.settings.lang);
  const cards: [Layout, string, string, string][] = [
    ["bijoy", "অ", s.bijoyName, s.bijoyDesc],
    ["avro", "a", s.avroName, s.avroDesc],
  ];
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 8, width: "100%" }} role="radiogroup" aria-label={s.chooseLayout}>
      {cards.map(([id, glyph, name, desc]) => {
        const on = boot.settings.layout === id;
        return (
          <div
            key={id}
            role="radio"
            aria-checked={on}
            tabIndex={0}
            className={"layout-card" + (on ? "" : " off")}
            onClick={() => api.setSettings({ layout: id })}
            onKeyDown={(e) => (e.key === "Enter" || e.key === " ") && api.setSettings({ layout: id })}
          >
            <div className={"app-glyph" + (id === "bijoy" ? " bn" : " mono")} style={{ width: 48, height: 48, fontSize: 24 }}>
              {glyph}
            </div>
            <div style={{ flex: 1, display: "flex", flexDirection: "column", gap: 4, textAlign: "left" }}>
              <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                <span style={{ fontSize: 15, fontWeight: 700 }}>{name}</span>
                {on && <span className="pill">{s.active}</span>}
              </div>
              <div className="sub">{desc}</div>
            </div>
            {extra?.(id)}
          </div>
        );
      })}
    </div>
  );
}

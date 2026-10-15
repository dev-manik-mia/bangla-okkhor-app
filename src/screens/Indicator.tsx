import { ModeDot } from "../components/ui";
import { api, type Bootstrap } from "../lib/api";
import { t } from "../lib/i18n";

export function Indicator({ boot }: { boot: Bootstrap }) {
  const s = t(boot.settings.lang);
  return (
    <button className="indicator" onClick={() => api.toggleMode()} title={s.toggleLang}>
      <ModeDot bangla={boot.bangla} size={24} />
      <span className="bn" style={{ fontWeight: 600 }}>
        {boot.bangla ? "বাংলা" : "English"}
      </span>
      <span className="sub" style={{ fontSize: 11 }}>
        {boot.bangla ? s.layoutName : "U.S."}
      </span>
    </button>
  );
}

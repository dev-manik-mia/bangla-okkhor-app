import { ModeDot } from "../components/ui";
import { api, type Bootstrap } from "../lib/api";
import { layoutLabel, t } from "../lib/i18n";

export function Indicator({ boot }: { boot: Bootstrap }) {
  const s = t(boot.settings.lang);
  return (
    <button
      className="indicator"
      onClick={() => (boot.permission.running ? api.toggleMode() : api.fixAccess())}
      title={boot.permission.running ? s.toggleLang : s.typingOff}
    >
      <ModeDot bangla={boot.bangla} size={24} />
      <span className="bn" style={{ fontWeight: 600 }}>
        {boot.bangla ? "বাংলা" : "English"}
      </span>
      <span className="sub" style={{ fontSize: 11 }}>
        {!boot.permission.running ? "⚠︎ " + s.fixAccess : boot.bangla ? layoutLabel(s, boot.settings.layout) : "U.S."}
      </span>
    </button>
  );
}

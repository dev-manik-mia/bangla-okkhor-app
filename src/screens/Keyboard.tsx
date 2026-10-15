import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";
import { KeyboardGrid } from "../components/KeyboardGrid";
import { Window } from "../components/ui";
import { api, shortcutLabel, type Bootstrap } from "../lib/api";
import { fill, t } from "../lib/i18n";

export function Keyboard({ boot }: { boot: Bootstrap }) {
  const s = t(boot.settings.lang);
  const [shift, setShift] = useState(false);
  const [pressed, setPressed] = useState<string | null>(null);
  const [text, setText] = useState("");
  const [copied, setCopied] = useState(false);
  const timer = useRef<number>();

  const flash = (code: string) => {
    setPressed(code);
    clearTimeout(timer.current);
    timer.current = window.setTimeout(() => setPressed(null), 140);
  };

  useEffect(() => {
    const un = listen<{ code: string }>("key", (e) => flash(e.payload.code));
    return () => {
      un.then((u) => u());
    };
  }, []);

  const press = (code: string) => {
    flash(code);
    if (code.startsWith("Shift")) return setShift((v) => !v);
    if (code === "CapsLock") return;
    api.virtualKey(code, shift);
    if (shift) setShift(false);
  };

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      document.execCommand("copy");
    }
    setCopied(true);
    setTimeout(() => setCopied(false), 1200);
  };

  const toggle = shortcutLabel(boot.settings.shortcuts.toggle, boot.platform);
  return (
    <Window title={s.kbTitle}>
      <div className="kb-body">
        <div className="kb-test">
          <textarea
            className="field"
            value={text}
            placeholder={s.testPh}
            spellCheck={false}
            onChange={(e) => setText(e.target.value)}
            onMouseDown={() => api.setKeyboardFocus(true)}
            style={{ height: 110, fontFamily: "var(--editor-font)", fontSize: "var(--editor-size)" }}
          />
          <div className="kb-test-actions">
            <button className="btn sm" onClick={copy}>
              {copied ? s.copied : s.copy}
            </button>
            <button className="btn sm" onClick={() => setText("")}>
              {s.clear}
            </button>
          </div>
        </div>
        <div className="kb-sugg" aria-live="polite">
          {boot.suggestions.length > 0 && (
            <>
              <span className="sub" style={{ padding: "0 8px", fontSize: 11 }}>
                {s.suggestions}
              </span>
              {boot.suggestions.map((w, i) => (
                <button key={w} className="sugg" onMouseDown={(e) => (e.preventDefault(), api.acceptSuggestion(w))}>
                  <span className="n">{i + 1}</span>
                  {w}
                </button>
              ))}
            </>
          )}
        </div>
        <KeyboardGrid
          layout={boot.layout}
          bangla={boot.bangla}
          shift={shift}
          pressed={pressed}
          spaceLabel={boot.bangla ? s.bn : s.en}
          onPress={press}
        />
        <div className="kb-foot sub">
          <span>
            {fill(s.kbHint, { toggle })} {boot.target.name && fill(s.kbHintTarget, { app: boot.target.name })}
          </span>
          <span>{s.layoutName}</span>
        </div>
      </div>
    </Window>
  );
}

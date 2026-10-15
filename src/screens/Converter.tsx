import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { Window } from "../components/ui";
import { api, type Bootstrap } from "../lib/api";
import { digits, t } from "../lib/i18n";

type Dir = "u2a" | "a2u";

export function Converter({ boot }: { boot: Bootstrap }) {
  const s = t(boot.settings.lang);
  const [dir, setDir] = useState<Dir>("u2a");
  const [input, setInput] = useState(s.sample);
  const [output, setOutput] = useState("");
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    let alive = true;
    api.convert(input, dir === "u2a" ? "ansi" : "unicode").then((o) => alive && setOutput(o));
    return () => {
      alive = false;
    };
  }, [input, dir]);

  // Another window can hand text over, e.g. the widget's Convert button.
  useEffect(() => {
    const un = listen<{ text: string; dir: Dir }>("converter-input", (e) => {
      setDir(e.payload.dir);
      setInput(e.payload.text);
    });
    return () => {
      un.then((u) => u());
    };
  }, []);

  const bnFont = "var(--bn-font)";
  const inFont = dir === "u2a" ? bnFont : "var(--mono)";
  const outFont = dir === "u2a" ? "var(--mono)" : bnFont;
  const copy = async () => {
    await navigator.clipboard.writeText(output).catch(() => {});
    setCopied(true);
    setTimeout(() => setCopied(false), 1200);
  };

  return (
    <Window title={s.convTitle}>
      <div className="conv">
        <div className="conv-pane">
          <div className="conv-head">
            <b>{dir === "u2a" ? s.uni : s.ansi}</b>
            <span className="sub">
              {digits(Array.from(input).length, boot.settings.lang)} {s.chars}
            </span>
          </div>
          <textarea className="field" spellCheck={false} value={input} onChange={(e) => setInput(e.target.value)} style={{ fontFamily: inFont }} />
        </div>
        <div style={{ display: "flex", alignItems: "center" }}>
          <button
            className="swap"
            title={s.swap}
            aria-label={s.swap}
            onClick={() => {
              setDir(dir === "u2a" ? "a2u" : "u2a");
              setInput(output);
            }}
          >
            ⇄
          </button>
        </div>
        <div className="conv-pane">
          <div className="conv-head">
            <b>{dir === "u2a" ? s.ansi : s.uni}</b>
          </div>
          <div className="conv-out" style={{ fontFamily: outFont }}>
            {output}
          </div>
        </div>
      </div>
      <div style={{ display: "flex", justifyContent: "flex-end", padding: "0 18px 18px" }}>
        <button className="btn primary" onClick={copy}>
          {copied ? s.copied : s.copyOut}
        </button>
      </div>
    </Window>
  );
}

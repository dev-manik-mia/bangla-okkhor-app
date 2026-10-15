import { openUrl } from "@tauri-apps/plugin-opener";
import { useEffect, useState } from "react";
import { Segmented } from "../components/ui";
import { api, shortcutLabel, type Bootstrap } from "../lib/api";
import { digits, fill, setupSteps, t } from "../lib/i18n";

const MAC_ACCESSIBILITY = "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility";

export function Onboarding({ boot }: { boot: Bootstrap }) {
  const st = boot.settings;
  const s = t(st.lang);
  const [step, setStep] = useState(0);
  const [perm, setPerm] = useState(boot.permission);
  const ready = perm.granted && perm.running;
  const toggle = shortcutLabel(st.shortcuts.toggle, boot.platform);

  useEffect(() => setPerm(boot.permission), [boot.permission]);

  // While on the permission step, poll until the hook is running, then move on.
  useEffect(() => {
    if (step !== 2) return;
    const id = window.setInterval(() => api.permissionStatus().then(setPerm), 1500);
    return () => clearInterval(id);
  }, [step]);
  useEffect(() => {
    if (step === 2 && ready && boot.platform === "macos") {
      const id = window.setTimeout(() => setStep(3), 800);
      return () => clearTimeout(id);
    }
  }, [step, ready, boot.platform]);

  const setup = setupSteps(s, boot.platform);
  const next = () => (step === 3 ? api.finishOnboarding() : setStep(step + 1));

  return (
    <div className="window onboard">
      <div className="onboard-top" data-tauri-drag-region>
        <span className="sub" style={{ fontSize: 11 }}>
          {s.step} {digits(step + 1, st.lang)} / {digits(4, st.lang)}
        </span>
      </div>
      <div className="onboard-body">
        {step === 0 && (
          <>
            <div className="app-glyph bn hero">অ</div>
            <div style={{ fontSize: 26, fontWeight: 700 }}>{s.wTitle}</div>
            <div className="sub" style={{ fontSize: 14, maxWidth: 360 }}>
              {s.wBody}
            </div>
          </>
        )}
        {step === 1 && (
          <>
            <div className="ob-title">{s.cTitle}</div>
            <div className="sub" style={{ maxWidth: 380 }}>
              {s.cBody}
            </div>
            <div className="layout-card" style={{ width: "100%", textAlign: "left", padding: 14 }}>
              <div className="app-glyph bn" style={{ width: 44, height: 44, fontSize: 22, borderRadius: 12 }}>
                অ
              </div>
              <div style={{ flex: 1 }}>
                <div style={{ fontWeight: 700 }}>{s.bijoyName}</div>
                <div className="sub" style={{ fontSize: 12 }}>
                  {s.bijoyDesc}
                </div>
              </div>
              <span style={{ color: "var(--acc)", fontWeight: 700 }}>✓</span>
            </div>
            <div className="card row" style={{ width: "100%" }}>
              <span>{s.startIn}</span>
              <Segmented value={st.startIn} options={[["bn", s.bn], ["en", s.en]]} onChange={(v) => api.setSettings({ startIn: v })} />
            </div>
          </>
        )}
        {step === 2 && (
          <>
            <div className="ob-title">{setup.title}</div>
            <div style={{ width: "100%", display: "flex", flexDirection: "column", gap: 8, textAlign: "left" }}>
              {setup.steps.map((line, i) => (
                <div className="card ob-step" key={i}>
                  <span className="ob-num">{digits(i + 1, st.lang)}</span>
                  <span className={line.includes("sudo") ? "selectable" : ""}>{fill(line, { toggle })}</span>
                </div>
              ))}
            </div>
            <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
              {boot.platform === "macos" && !ready && (
                <button
                  className="btn"
                  onClick={() => {
                    api.requestPermission().then(setPerm);
                    openUrl(MAC_ACCESSIBILITY).catch(() => {});
                  }}
                >
                  {s.macOpen}
                </button>
              )}
              {boot.platform === "linux" && !ready && (
                <button className="btn" onClick={() => api.requestPermission().then(setPerm)}>
                  {s.checkAgain}
                </button>
              )}
              <span className="sub" style={{ fontSize: 12, color: ready ? "var(--acc)" : undefined }}>
                {ready ? "✓ " + s.permGranted : boot.platform !== "windows" ? s.permWaiting : ""}
              </span>
            </div>
            {!ready && perm.message && boot.platform === "linux" && (
              <div className="sub selectable" style={{ fontSize: 11 }}>
                {perm.message}
              </div>
            )}
          </>
        )}
        {step === 3 && (
          <>
            <div className="done">✓</div>
            <div style={{ fontSize: 24, fontWeight: 700 }}>{s.dTitle}</div>
            <div className="sub" style={{ fontSize: 14 }}>
              {s.dBody}
            </div>
            <div style={{ display: "flex", gap: 6, alignItems: "center" }} className="mono">
              {["h", "f", "Q", "V", "f"].map((k, i) => (
                <span className="keycap" key={i}>
                  {k}
                </span>
              ))}
              <span className="sub" style={{ padding: "0 6px" }}>
                →
              </span>
              <span className="bn" style={{ fontSize: 24, fontWeight: 600 }}>
                বাংলা
              </span>
            </div>
          </>
        )}
      </div>
      <div className="onboard-foot">
        <div style={{ display: "flex", gap: 5 }}>
          {[0, 1, 2, 3].map((d) => (
            <span key={d} className="dot" style={{ width: d === step ? 18 : 6, background: d === step ? "var(--acc)" : "var(--key)" }} />
          ))}
        </div>
        <div style={{ display: "flex", gap: 8 }}>
          {step > 0 && (
            <button className="btn" onClick={() => setStep(step - 1)}>
              {s.back}
            </button>
          )}
          <button className="btn primary" onClick={next}>
            {step === 3 ? s.finish : s.next}
          </button>
        </div>
      </div>
    </div>
  );
}

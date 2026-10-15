import type { ReactNode } from "react";

export function Toggle({ on, onChange, label }: { on: boolean; onChange: (v: boolean) => void; label: string }) {
  return (
    <button type="button" role="switch" aria-checked={on} aria-label={label} className={"toggle" + (on ? " on" : "")} onClick={() => onChange(!on)}>
      <span />
    </button>
  );
}

export function Segmented<T extends string>({
  value,
  options,
  onChange,
  fill,
}: {
  value: T;
  options: [T, ReactNode][];
  onChange: (v: T) => void;
  fill?: boolean;
}) {
  return (
    <div className={"seg" + (fill ? " fill" : "")} role="radiogroup">
      {options.map(([v, label]) => (
        <button key={v} type="button" role="radio" aria-checked={value === v} className={value === v ? "on" : ""} onClick={() => onChange(v)}>
          {label}
        </button>
      ))}
    </div>
  );
}

export function ToggleRow({ label, on, onChange }: { label: string; on: boolean; onChange: (v: boolean) => void }) {
  return (
    <div className="row">
      <span>{label}</span>
      <Toggle on={on} onChange={onChange} label={label} />
    </div>
  );
}

export function ModeDot({ bangla, size = 22 }: { bangla: boolean; size?: number }) {
  return (
    <span className={"mode-dot" + (bangla ? " bn" : "")} style={{ width: size, height: size, fontSize: Math.round(size * 0.55) }}>
      {bangla ? "অ" : "A"}
    </span>
  );
}

/** Shell for regular windows: on macOS draws a drag-able title row under the traffic lights. */
export function Window({ title, children }: { title?: string; children: ReactNode }) {
  return (
    <div className="window">
      {title !== undefined && (
        <div className="titlebar" data-tauri-drag-region>
          <span data-tauri-drag-region>{title}</span>
        </div>
      )}
      {children}
    </div>
  );
}

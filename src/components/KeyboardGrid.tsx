import type { LayoutKey } from "../lib/api";

const ROWS = [
  ["Backquote", "Digit1", "Digit2", "Digit3", "Digit4", "Digit5", "Digit6", "Digit7", "Digit8", "Digit9", "Digit0", "Minus", "Equal", "Backspace"],
  ["Tab", "KeyQ", "KeyW", "KeyE", "KeyR", "KeyT", "KeyY", "KeyU", "KeyI", "KeyO", "KeyP", "BracketLeft", "BracketRight", "Backslash"],
  ["CapsLock", "KeyA", "KeyS", "KeyD", "KeyF", "KeyG", "KeyH", "KeyJ", "KeyK", "KeyL", "Semicolon", "Quote", "Enter"],
  ["ShiftLeft", "KeyZ", "KeyX", "KeyC", "KeyV", "KeyB", "KeyN", "KeyM", "Comma", "Period", "Slash", "ShiftRight"],
  ["Space"],
];

const SPECIAL: Record<string, [string, number]> = {
  Backspace: ["⌫", 1.6],
  Tab: ["⇥", 1.5],
  CapsLock: ["⇪", 1.8],
  Enter: ["↩", 1.9],
  ShiftLeft: ["⇧", 2.4],
  ShiftRight: ["⇧", 2.4],
};

const isMark = (s: string) => {
  const n = s.codePointAt(0) ?? 0;
  return (n >= 0x981 && n <= 0x983) || (n >= 0x9bc && n <= 0x9d7);
};
const show = (s: string) => (isMark(s) ? "◌" + s : s);

export function KeyboardGrid({
  layout,
  bangla,
  shift,
  pressed,
  spaceLabel,
  onPress,
}: {
  layout: LayoutKey[];
  bangla: boolean;
  shift: boolean;
  pressed: string | null;
  spaceLabel: string;
  onPress: (code: string) => void;
}) {
  const byCode = new Map(layout.map((k) => [k.code, k]));
  return (
    <div className="kb">
      {ROWS.map((row, i) => (
        <div className="kb-row" key={i}>
          {row.map((code) => {
            const sp = SPECIAL[code];
            const k = byCode.get(code);
            const on = pressed === code || (code.startsWith("Shift") && shift);
            let label = "",
              alt = "",
              lat = "";
            if (sp) label = sp[0];
            else if (code === "Space") label = spaceLabel;
            else if (k) {
              if (bangla) {
                lat = k.lat.toUpperCase();
                label = show(shift ? k.bnShift : k.bn);
                alt = show(shift ? k.bn : k.bnShift);
              } else {
                label = shift ? k.latShift : k.lat;
              }
            }
            return (
              <div
                key={code}
                className={"key" + (sp ? " mod" : "") + (on ? " on" : "") + (code === "Space" ? " space" : "")}
                style={{ flex: code === "Space" ? "1 1 0" : `${sp ? sp[1] : 1} 1 0` }}
                onMouseDown={(e) => {
                  e.preventDefault();
                  onPress(code);
                }}
              >
                {lat && <span className="lat">{lat}</span>}
                {alt && <span className="alt">{alt}</span>}
                <span className={"main" + (sp ? " sym" : "") + (code === "Space" ? " small" : "")}>{label}</span>
              </div>
            );
          })}
        </div>
      ))}
    </div>
  );
}

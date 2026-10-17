import { getCurrentWindow, PhysicalPosition } from "@tauri-apps/api/window";
import { useRef, type PointerEvent } from "react";
import { api } from "./api";

/**
 * Moves the window by following the pointer. The widget and the mode pill are
 * non-activating panels, where the OS-level drag (`startDragging`) is ignored
 * on macOS, so we position the window ourselves. Pointer capture keeps the moves coming even
 * when the cursor runs ahead of the window.
 */
export function useDrag(win: ReturnType<typeof getCurrentWindow>) {
  const start = useRef<{ x: number; y: number; sx: number; sy: number; scale: number } | null>(null);
  const pending = useRef<{ x: number; y: number } | null>(null);
  const frame = useRef(0);

  const down = async (e: PointerEvent<HTMLElement>) => {
    if (e.button !== 0 || (e.target as HTMLElement).closest("button")) return;
    e.currentTarget.setPointerCapture(e.pointerId);
    const { screenX: sx, screenY: sy } = e;
    const [pos, scale] = await Promise.all([win.outerPosition(), win.scaleFactor()]);
    start.current = { x: pos.x, y: pos.y, sx, sy, scale };
  };

  const move = (e: PointerEvent<HTMLElement>) => {
    const s = start.current;
    if (!s) return;
    pending.current = { x: Math.round(s.x + (e.screenX - s.sx) * s.scale), y: Math.round(s.y + (e.screenY - s.sy) * s.scale) };
    if (frame.current) return;
    frame.current = requestAnimationFrame(() => {
      frame.current = 0;
      const p = pending.current;
      if (p) win.setPosition(new PhysicalPosition(p.x, p.y)).catch(() => {});
    });
  };

  const up = (e: PointerEvent<HTMLElement>) => {
    if (!start.current) return;
    start.current = null;
    if (e.currentTarget.hasPointerCapture(e.pointerId)) e.currentTarget.releasePointerCapture(e.pointerId);
    api.saveSettings();
  };

  return { down, move, up };
}

/** Key click sounds, synthesized with Web Audio (no sample files to ship). */

export type SoundKind = "soft" | "mechanical" | "typewriter" | "pop" | "wood";

export const SOUND_KINDS: SoundKind[] = ["soft", "mechanical", "typewriter", "pop", "wood"];

let ctx: AudioContext | undefined;
let noise: AudioBuffer | undefined;

function audio(): AudioContext {
  ctx ??= new AudioContext();
  if (ctx.state === "suspended") void ctx.resume();
  return ctx;
}

/** Half a second of white noise, reused by every click. */
function noiseBuffer(ac: AudioContext): AudioBuffer {
  if (!noise) {
    noise = ac.createBuffer(1, Math.floor(ac.sampleRate * 0.5), ac.sampleRate);
    const d = noise.getChannelData(0);
    for (let i = 0; i < d.length; i++) d[i] = Math.random() * 2 - 1;
  }
  return noise;
}

/** A gain node that rises to `peak` in 2 ms and fades out over `decay` seconds. */
function envelope(ac: AudioContext, t: number, peak: number, decay: number): GainNode {
  const g = ac.createGain();
  g.gain.setValueAtTime(0.0001, t);
  g.gain.linearRampToValueAtTime(peak, t + 0.002);
  g.gain.exponentialRampToValueAtTime(0.0001, t + decay);
  g.connect(ac.destination);
  return g;
}

function burst(ac: AudioContext, t: number, filter: BiquadFilterType, freq: number, q: number, peak: number, decay: number) {
  const src = ac.createBufferSource();
  src.buffer = noiseBuffer(ac);
  const f = ac.createBiquadFilter();
  f.type = filter;
  f.frequency.value = freq;
  f.Q.value = q;
  src.connect(f).connect(envelope(ac, t, peak, decay));
  src.start(t, Math.random() * 0.4, decay + 0.02);
}

function tone(ac: AudioContext, t: number, type: OscillatorType, from: number, to: number, peak: number, decay: number) {
  const osc = ac.createOscillator();
  osc.type = type;
  osc.frequency.setValueAtTime(from, t);
  if (to !== from) osc.frequency.exponentialRampToValueAtTime(to, t + decay * 0.8);
  osc.connect(envelope(ac, t, peak, decay));
  osc.start(t);
  osc.stop(t + decay + 0.02);
}

export function playKeySound(kind: SoundKind) {
  const ac = audio();
  const t = ac.currentTime + 0.001;
  // A little pitch variation so fast typing doesn't sound like a machine gun.
  const v = 0.94 + Math.random() * 0.12;
  switch (kind) {
    case "mechanical":
      burst(ac, t, "bandpass", 3800 * v, 1.2, 0.35, 0.018);
      tone(ac, t, "sine", 170 * v, 95 * v, 0.45, 0.06);
      break;
    case "typewriter":
      burst(ac, t, "bandpass", 2400 * v, 0.9, 0.6, 0.03);
      tone(ac, t, "triangle", 1900 * v, 1900 * v, 0.035, 0.09);
      tone(ac, t, "sine", 120 * v, 80 * v, 0.3, 0.04);
      break;
    case "pop":
      tone(ac, t, "sine", 320 * v, 880 * v, 0.3, 0.07);
      break;
    case "wood":
      tone(ac, t, "sine", 880 * v, 860 * v, 0.28, 0.08);
      tone(ac, t, "sine", 1960 * v, 1900 * v, 0.07, 0.04);
      burst(ac, t, "lowpass", 2500, 0.7, 0.12, 0.012);
      break;
    case "soft":
    default:
      burst(ac, t, "lowpass", 1400 * v, 0.7, 0.35, 0.035);
      tone(ac, t, "sine", 240 * v, 180 * v, 0.12, 0.03);
      break;
  }
}

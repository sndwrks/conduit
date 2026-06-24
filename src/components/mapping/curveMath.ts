import type { CalibrationPoint } from "@/types";

export function linearDistribute(min: number, max: number, n: number): number[] {
  if (n <= 1) return [min];
  const step = (max - min) / (n - 1);
  return Array.from({ length: n }, (_, i) => min + step * i);
}

function linearInterpolate(x: number, points: CalibrationPoint[]): number {
  const segments = points.length - 1;
  let seg = segments - 1;
  for (let i = 0; i < segments - 1; i++) {
    if (x < points[i + 1].input) {
      seg = i;
      break;
    }
  }
  const p0 = points[seg];
  const p1 = points[seg + 1];
  const range = p1.input - p0.input;
  if (Math.abs(range) < Number.EPSILON) return p0.output;
  const t = (x - p0.input) / range;
  return p0.output + t * (p1.output - p0.output);
}

function monotoneCubic(x: number, points: CalibrationPoint[]): number {
  const n = points.length;
  const segments = n - 1;

  const delta: number[] = [];
  for (let i = 0; i < segments; i++) {
    const dx = points[i + 1].input - points[i].input;
    delta.push(Math.abs(dx) < Number.EPSILON ? 0 : (points[i + 1].output - points[i].output) / dx);
  }

  const m: number[] = new Array(n).fill(0);
  m[0] = delta[0];
  m[segments] = delta[segments - 1];
  for (let i = 1; i < segments; i++) {
    m[i] = delta[i - 1] * delta[i] > 0 ? (delta[i - 1] + delta[i]) / 2 : 0;
  }

  for (let i = 0; i < segments; i++) {
    if (Math.abs(delta[i]) < Number.EPSILON) {
      m[i] = 0;
      m[i + 1] = 0;
    } else {
      const alpha = m[i] / delta[i];
      const beta = m[i + 1] / delta[i];
      const s = alpha * alpha + beta * beta;
      if (s > 9) {
        const tau = 3 / Math.sqrt(s);
        m[i] = tau * alpha * delta[i];
        m[i + 1] = tau * beta * delta[i];
      }
    }
  }

  let seg = segments - 1;
  for (let i = 0; i < segments - 1; i++) {
    if (x < points[i + 1].input) {
      seg = i;
      break;
    }
  }

  const h = points[seg + 1].input - points[seg].input;
  if (Math.abs(h) < Number.EPSILON) return points[seg].output;
  const t = (x - points[seg].input) / h;
  const t2 = t * t;
  const t3 = t2 * t;
  const h00 = 2 * t3 - 3 * t2 + 1;
  const h10 = t3 - 2 * t2 + t;
  const h01 = -2 * t3 + 3 * t2;
  const h11 = t3 - t2;

  return (
    h00 * points[seg].output +
    h10 * h * m[seg] +
    h01 * points[seg + 1].output +
    h11 * h * m[seg + 1]
  );
}

export function interpolateCalibrated(
  x: number,
  points: CalibrationPoint[],
  smoothing: number,
): number {
  if (points.length === 0) return 0;
  if (points.length === 1) return points[0].output;

  const first = points[0];
  const last = points[points.length - 1];
  if (x <= first.input) return first.output;
  if (x >= last.input) return last.output;

  const linear = linearInterpolate(x, points);
  if (points.length === 2) return linear;

  const s = Math.max(0, Math.min(1, smoothing));
  const cubic = monotoneCubic(x, points);
  return s * cubic + (1 - s) * linear;
}

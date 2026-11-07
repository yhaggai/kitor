import { useEffect, useRef } from "react";

type Point = { x: number; y: number };

// Interpolate a closed contour through the ribbon edges, rather than drawing
// straight segments. The moving waves travel from the mouth towards the top.
function contour(points: Point[]) {
  const n = points.length;
  const xy = (p: Point) => `${p.x.toFixed(1)} ${p.y.toFixed(1)}`;
  let path = `M ${xy(points[0])}`;
  for (let i = 0; i < n; i++) {
    const a = points[(i + n - 1) % n];
    const b = points[i];
    const c = points[(i + 1) % n];
    const d = points[(i + 2) % n];
    path += ` C ${xy({ x: b.x + (c.x - a.x) / 6, y: b.y + (c.y - a.y) / 6 })} ${xy({ x: c.x - (d.x - b.x) / 6, y: c.y - (d.y - b.y) / 6 })} ${xy(c)}`;
  }
  return `${path} Z`;
}

function ribbon(time: number, lane: number) {
  const steps = 36;
  const centers: Point[] = [];
  const widths: number[] = [];
  for (let i = 0; i <= steps; i++) {
    const u = i / steps;
    const phase = u * 10.5 - time * 1.7 + lane * 1.65;
    const spread = Math.sin(u * Math.PI * 0.94) * Math.sqrt(u);
    centers.push({
      x:
        1398 +
        u * (lane === 2 ? -35 : 110) +
        spread * (135 * Math.sin(phase) + 32 * Math.sin(phase * 1.63 + time * 0.19)),
      y: 485 - 445 * u + spread * 25 * Math.cos(phase),
    });
    const taper = Math.sin(Math.PI * (0.06 + u * 0.94)) ** 0.7;
    widths.push(
      (lane === 0 ? 31 : lane === 1 ? 22 : 10) * taper * (0.76 + 0.24 * Math.sin(phase - 0.8))
    );
  }
  const left: Point[] = [];
  const right: Point[] = [];
  for (let i = 0; i <= steps; i++) {
    const before = centers[Math.max(0, i - 1)];
    const after = centers[Math.min(steps, i + 1)];
    const dx = after.x - before.x;
    const dy = after.y - before.y;
    const length = Math.hypot(dx, dy);
    const nx = (-dy / length) * widths[i];
    const ny = (dx / length) * widths[i];
    left.push({ x: centers[i].x + nx, y: centers[i].y + ny });
    right.push({ x: centers[i].x - nx, y: centers[i].y - ny });
  }
  return contour([...left, ...right.reverse()]);
}

export function BrandSteam({ id, active }: { id: string; active: boolean }) {
  const svg = useRef<SVGSVGElement>(null);
  const fade = useRef<Animation | null>(null);

  useEffect(() => {
    const paths = svg.current?.querySelectorAll<SVGPathElement>(".kt-steam-ribbon");
    if (!paths) return;
    let frame = 0;
    const draw = (now: number) => {
      for (const path of paths) {
        path.setAttribute("d", ribbon(now / 1000, Number(path.dataset.lane)));
      }
      frame = requestAnimationFrame(draw);
    };
    draw(performance.now());
    return () => {
      cancelAnimationFrame(frame);
      fade.current?.cancel();
    };
  }, []);

  useEffect(() => {
    const element = svg.current;
    if (!element) return;
    const opacity = getComputedStyle(element).opacity;
    fade.current?.cancel();
    fade.current = element.animate([{ opacity }, { opacity: active ? 1 : 0 }], {
      duration: active ? 450 : 550,
      easing: "ease",
      fill: "forwards",
    });
  }, [active]);

  return (
    <svg
      ref={svg}
      className="kt-brand-steam-live"
      viewBox="0 0 1774 887"
      aria-hidden="true"
      focusable="false"
    >
      <defs>
        <linearGradient id={`${id}-fade`} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stopColor="white" stopOpacity="0" />
          <stop offset="0.25" stopColor="white" stopOpacity="0.25" />
          <stop offset="0.6" stopColor="white" />
          <stop offset="1" stopColor="white" />
        </linearGradient>
        <mask id={`${id}-mask`} maskUnits="userSpaceOnUse" x="1170" y="12" width="570" height="468">
          <rect x="1170" y="12" width="570" height="468" fill={`url(#${id}-fade)`} />
        </mask>
      </defs>
      <g mask={`url(#${id}-mask)`}>
        <path className="kt-steam-ribbon" data-lane="2" fill="#133a57" opacity="0.4" />
        <path className="kt-steam-ribbon" data-lane="1" fill="#feca66" />
        <path className="kt-steam-ribbon" data-lane="0" fill="#133a57" />
      </g>
    </svg>
  );
}

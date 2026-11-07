import { useEffect, useId, useMemo, useRef } from "react";
import logo from "../assets/kitor-logo.svg?raw";

/** Original artwork, with the plume anchored to the stationary chimney rim. */
export function BrandLogo({ className = "" }: { className?: string }) {
  const id = useId();
  const artwork = useMemo(() => logo.replace(/kitor-logo-/g, `${id}-`), [id]);
  const root = useRef<HTMLDivElement>(null);
  const animation = useRef<Animation | null>(null);

  useEffect(() => {
    const motion = window.matchMedia("(prefers-reduced-motion: reduce)");
    const reset = () => {
      animation.current?.cancel();
      animation.current = null;
    };
    const onChange = () => {
      if (motion.matches) reset();
    };
    motion.addEventListener("change", onChange);
    return () => {
      motion.removeEventListener("change", onChange);
      reset();
    };
  }, []);

  function moveSteam(hovered: boolean) {
    if (!hovered && !animation.current) return;
    const steam = root.current?.querySelector<SVGGElement>(".kt-brand-steam");
    if (!steam || window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;

    // Start from the visible pose, including when a quick re-entry interrupts settling.
    const current = getComputedStyle(steam).transform;
    animation.current?.cancel();
    const frames = hovered
      ? [
          { transform: current },
          { transform: "skewX(-3.5deg) scale(1.018, 1.065)", offset: 0.28 },
          { transform: "skewX(2.4deg) scale(0.99, 1.015)", offset: 0.58 },
          { transform: "skewX(-1deg) scale(1.012, 1.03)", offset: 0.82 },
          { transform: current },
        ]
      : [{ transform: current }, { transform: "none" }];
    animation.current = steam.animate(
      frames.map((frame) => ({ ...frame, easing: hovered ? "ease-in-out" : "linear" })),
      {
        duration: hovered ? 3600 : 650,
        iterations: hovered ? Number.POSITIVE_INFINITY : 1,
        easing: hovered ? "linear" : "cubic-bezier(0.22, 1, 0.36, 1)",
      }
    );
  }

  return (
    <div
      ref={root}
      className={`kt-brand-logo ${className}`}
      onPointerEnter={(event) => {
        if (event.pointerType !== "touch") moveSteam(true);
      }}
      onPointerLeave={() => moveSteam(false)}
      onPointerCancel={() => moveSteam(false)}
      // biome-ignore lint/security/noDangerouslySetInnerHtml: Trusted, bundled SVG artwork; never user-supplied markup.
      dangerouslySetInnerHTML={{ __html: artwork }}
    />
  );
}

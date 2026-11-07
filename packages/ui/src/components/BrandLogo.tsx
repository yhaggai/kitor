import { useEffect, useId, useMemo, useRef, useState } from "react";
import logo from "../assets/kitor-logo.svg?raw";
import { BrandSteam } from "./BrandSteam";

/** Preserve the original logo at rest; emit illustrated steam on pointer hover. */
export function BrandLogo({ className = "" }: { className?: string }) {
  const id = useId();
  const artwork = useMemo(() => logo.replace(/kitor-logo-/g, `${id}-`), [id]);
  const [active, setActive] = useState(false);
  const [running, setRunning] = useState(false);
  const stopTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  useEffect(() => {
    const motion = window.matchMedia("(prefers-reduced-motion: reduce)");
    const reset = () => {
      if (!motion.matches) return;
      clearTimeout(stopTimer.current);
      setActive(false);
      setRunning(false);
    };
    motion.addEventListener("change", reset);
    return () => {
      motion.removeEventListener("change", reset);
      clearTimeout(stopTimer.current);
    };
  }, []);

  function startSteam() {
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    clearTimeout(stopTimer.current);
    setRunning(true);
    setActive(true);
  }

  function stopSteam() {
    setActive(false);
    clearTimeout(stopTimer.current);
    // Let emitted curls dissolve while the original artwork returns, then stop
    // all animation work. Re-entering during this fade preserves the live flow.
    stopTimer.current = setTimeout(() => setRunning(false), 600);
  }

  return (
    <div
      className={`kt-brand-logo ${className}`}
      data-steaming={active || undefined}
      onPointerEnter={(event) => {
        if (event.pointerType !== "touch") startSteam();
      }}
      onPointerLeave={stopSteam}
      onPointerCancel={stopSteam}
    >
      <div
        className="kt-brand-artwork"
        // biome-ignore lint/security/noDangerouslySetInnerHtml: Trusted, bundled SVG artwork; never user-supplied markup.
        dangerouslySetInnerHTML={{ __html: artwork }}
      />
      {running && <BrandSteam id={`${id}-live`} active={active} />}
    </div>
  );
}

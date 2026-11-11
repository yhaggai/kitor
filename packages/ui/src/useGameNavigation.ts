import { useLayoutEffect, useRef, useState } from "react";
import type { GameSummary } from "./types";

/** FLIP the actual cover between grid and detail. No View Transition API dependency (WebKit safe). */
export function useGameNavigation(resetKey?: number) {
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const previousResetKey = useRef(resetKey);
  const root = useRef<HTMLDivElement>(null);
  const origin = useRef<{ button: HTMLButtonElement; rect: DOMRect; scroll: number } | null>(null);
  const returning = useRef<DOMRect | null>(null);
  const open = (game: GameSummary, button: HTMLButtonElement) => {
    origin.current = {
      button,
      rect: (button.querySelector("[data-game-cover]") || button).getBoundingClientRect(),
      scroll: window.scrollY,
    };
    setSelectedId(game.id);
  };
  const back = () => {
    returning.current =
      root.current?.querySelector(".kt-details [data-game-cover]")?.getBoundingClientRect() || null;
    setSelectedId(null);
  };
  useLayoutEffect(() => {
    // Sidebar navigation opens the gallery without restoring a previous game's
    // focus, scroll position, or cover animation (including in hidden tabs).
    if (previousResetKey.current !== resetKey) {
      previousResetKey.current = resetKey;
      origin.current = null;
      returning.current = null;
      setSelectedId(null);
      return;
    }
    let animation: Animation | undefined;
    const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    const cover = selectedId
      ? root.current?.querySelector<HTMLElement>(".kt-details [data-game-cover]")
      : origin.current?.button.querySelector<HTMLElement>("[data-game-cover]");
    if (selectedId) {
      window.scrollTo({ top: 0, behavior: "instant" });
      root.current
        ?.querySelector<HTMLElement>("[data-detail-heading]")
        ?.focus({ preventScroll: true });
    } else if (origin.current) {
      window.scrollTo({ top: origin.current.scroll, behavior: "instant" });
      if (origin.current.button.isConnected) origin.current.button.focus({ preventScroll: true });
    }
    const from = selectedId ? origin.current?.rect : returning.current;
    if (cover && from && !reduced) {
      const to = cover.getBoundingClientRect();
      if (to.width && to.height)
        animation = cover.animate(
          [
            {
              transformOrigin: "top left",
              transform: `translate(${from.left - to.left}px, ${from.top - to.top}px) scale(${from.width / to.width}, ${from.height / to.height})`,
            },
            { transformOrigin: "top left", transform: "none" },
          ],
          { duration: 380, easing: "cubic-bezier(.22,1,.36,1)" }
        );
    }
    return () => animation?.cancel();
  }, [selectedId, resetKey]);
  return { root, selectedId, open, back, select: setSelectedId };
}

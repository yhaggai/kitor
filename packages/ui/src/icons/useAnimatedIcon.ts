import { useAnimation } from "motion/react";
import { type HTMLAttributes, type Ref, useImperativeHandle } from "react";

export type AnimatedIconHandle = {
  startAnimation: () => void;
  stopAnimation: () => void;
};

export type AnimatedIconProps = HTMLAttributes<HTMLDivElement> & {
  size?: number;
};

const prefersReducedMotion = () =>
  typeof window !== "undefined" && window.matchMedia("(prefers-reduced-motion: reduce)").matches;

/** Parent hover (e.g. sidebar button) drives start/stop via ref. */
export function useAnimatedIcon(ref: Ref<AnimatedIconHandle> | undefined) {
  const controls = useAnimation();

  useImperativeHandle(ref, () => ({
    startAnimation: () => {
      if (!prefersReducedMotion()) void controls.start("animate");
    },
    stopAnimation: () => {
      void controls.start("normal");
    },
  }));

  return { controls };
}

export function iconClassName(className?: string) {
  return ["kt-animated-icon", className].filter(Boolean).join(" ");
}

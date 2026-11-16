import type { Variants } from "motion/react";
import { motion } from "motion/react";
import { forwardRef } from "react";
import {
  type AnimatedIconHandle,
  type AnimatedIconProps,
  iconClassName,
  useAnimatedIcon,
} from "./useAnimatedIcon";

const HAND_VARIANTS: Variants = {
  normal: {
    rotate: 0,
    originX: "0%",
    originY: "100%",
    transition: { duration: 0.6, ease: [0.4, 0, 0.2, 1] },
  },
  animate: {
    rotate: 300,
    originX: "0%",
    originY: "100%",
    transition: { delay: 0.1, duration: 0.6, ease: [0.4, 0, 0.2, 1] },
  },
};

const BUTTON_VARIANTS: Variants = {
  normal: { scale: 1, y: 0 },
  animate: {
    scale: [0.9, 1],
    y: [0, 1, 0],
    transition: { duration: 0.3, ease: [0.4, 0, 0.2, 1] },
  },
};

export const TimerIcon = forwardRef<AnimatedIconHandle, AnimatedIconProps>(
  ({ className, size = 28, ...props }, ref) => {
    const { controls } = useAnimatedIcon(ref);
    return (
      <div className={iconClassName(className)} {...props}>
        <svg
          fill="none"
          height={size}
          stroke="currentColor"
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth="2"
          viewBox="0 0 24 24"
          width={size}
          xmlns="http://www.w3.org/2000/svg"
          aria-hidden="true"
        >
          <motion.line
            animate={controls}
            variants={BUTTON_VARIANTS}
            x1="10"
            x2="14"
            y1="2"
            y2="2"
          />
          <motion.line
            animate={controls}
            initial="normal"
            variants={HAND_VARIANTS}
            x1="12"
            x2="15"
            y1="14"
            y2="11"
          />
          <circle cx="12" cy="14" r="8" />
        </svg>
      </div>
    );
  }
);

TimerIcon.displayName = "TimerIcon";

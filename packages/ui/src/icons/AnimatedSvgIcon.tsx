import type { Variants } from "motion/react";
import { motion } from "motion/react";
import { forwardRef, type ReactNode } from "react";
import {
  type AnimatedIconHandle,
  type AnimatedIconProps,
  iconClassName,
  useAnimatedIcon,
} from "./useAnimatedIcon";

export const wiggleVariants: Variants = {
  normal: { rotate: 0, y: 0 },
  animate: {
    rotate: [0, -8, 6, -3, 0],
    y: [0, -1, 0],
    transition: { duration: 0.45, ease: "easeInOut" },
  },
};

export const bounceVariants: Variants = {
  normal: { y: 0, scale: 1 },
  animate: {
    y: [0, -3, 0],
    scale: [1, 1.08, 1],
    transition: { duration: 0.4, ease: "easeInOut" },
  },
};

export const floatVariants: Variants = {
  normal: { y: 0 },
  animate: {
    y: [0, -4, -1, -3, 0],
    transition: { duration: 0.7, ease: "easeInOut" },
  },
};

export const spinNudgeVariants: Variants = {
  normal: { rotate: 0 },
  animate: {
    rotate: [0, 20, -12, 8, 0],
    transition: { duration: 0.55, ease: "easeInOut" },
  },
};

export function createMotionSvgIcon(
  displayName: string,
  paths: ReactNode,
  variants: Variants = wiggleVariants
) {
  const Icon = forwardRef<AnimatedIconHandle, AnimatedIconProps>(
    ({ className, size = 28, ...props }, ref) => {
      const { controls } = useAnimatedIcon(ref);
      return (
        <div className={iconClassName(className)} {...props}>
          <motion.svg
            animate={controls}
            fill="none"
            height={size}
            stroke="currentColor"
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth="2"
            variants={variants}
            viewBox="0 0 24 24"
            width={size}
            xmlns="http://www.w3.org/2000/svg"
            aria-hidden="true"
          >
            {paths}
          </motion.svg>
        </div>
      );
    }
  );
  Icon.displayName = displayName;
  return Icon;
}

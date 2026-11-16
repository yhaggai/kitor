import type { Variants } from "motion/react";
import { motion } from "motion/react";
import { forwardRef } from "react";
import {
  type AnimatedIconHandle,
  type AnimatedIconProps,
  iconClassName,
  useAnimatedIcon,
} from "./useAnimatedIcon";

/** Lucide `library` paths — no upstream animated variant, so we animate in-house. */
const SPINE_VARIANTS: Variants = {
  normal: { y: 0 },
  animate: (index: number) => ({
    y: [0, -2, 0],
    transition: {
      duration: 0.45,
      delay: index * 0.06,
      ease: "easeInOut",
    },
  }),
};

const SPINES = ["m16 6 4 14", "M12 6v14", "M8 8v12", "M4 4v16"] as const;

export const LibraryIcon = forwardRef<AnimatedIconHandle, AnimatedIconProps>(
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
          {SPINES.map((d, index) => (
            <motion.path
              key={d}
              animate={controls}
              custom={index}
              d={d}
              variants={SPINE_VARIANTS}
            />
          ))}
        </svg>
      </div>
    );
  }
);

LibraryIcon.displayName = "LibraryIcon";

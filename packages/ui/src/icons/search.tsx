import { motion } from "motion/react";
import { forwardRef } from "react";
import {
  type AnimatedIconHandle,
  type AnimatedIconProps,
  iconClassName,
  useAnimatedIcon,
} from "./useAnimatedIcon";

export const SearchIcon = forwardRef<AnimatedIconHandle, AnimatedIconProps>(
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
          transition={{ duration: 1, bounce: 0.3 }}
          variants={{
            normal: { x: 0, y: 0 },
            animate: { x: [0, 0, -3, 0], y: [0, -4, 0, 0] },
          }}
          viewBox="0 0 24 24"
          width={size}
          xmlns="http://www.w3.org/2000/svg"
          aria-hidden="true"
        >
          <circle cx="11" cy="11" r="8" />
          <path d="m21 21-4.3-4.3" />
        </motion.svg>
      </div>
    );
  }
);

SearchIcon.displayName = "SearchIcon";

import type { Variants } from "motion/react";
import { motion, useAnimation } from "motion/react";
import { forwardRef, useImperativeHandle } from "react";
import { type AnimatedIconHandle, type AnimatedIconProps, iconClassName } from "./useAnimatedIcon";

const PATH_VARIANTS: Variants = {
  normal: { opacity: 1, pathLength: 1, pathOffset: 0 },
  animate: (custom: number) => ({
    opacity: [0, 1],
    pathLength: [0, 1],
    pathOffset: [1, 0],
    transition: {
      opacity: { duration: 0.01, delay: custom * 0.1 },
      pathLength: {
        type: "spring",
        duration: 0.5,
        bounce: 0,
        delay: custom * 0.1,
      },
    },
  }),
};

const prefersReducedMotion = () =>
  typeof window !== "undefined" && window.matchMedia("(prefers-reduced-motion: reduce)").matches;

export const LanguagesIcon = forwardRef<AnimatedIconHandle, AnimatedIconProps>(
  ({ className, size = 28, ...props }, ref) => {
    const svgControls = useAnimation();
    const pathControls = useAnimation();

    useImperativeHandle(ref, () => ({
      startAnimation: () => {
        if (prefersReducedMotion()) return;
        void svgControls.start("animate");
        void pathControls.start("animate");
      },
      stopAnimation: () => {
        void svgControls.start("normal");
        void pathControls.start("normal");
      },
    }));

    return (
      <div className={iconClassName(className)} {...props}>
        <motion.svg
          animate={svgControls}
          fill="none"
          height={size}
          stroke="currentColor"
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth="2"
          variants={{
            normal: { opacity: 1 },
            animate: {
              opacity: 1,
              transition: { staggerChildren: 0.1, delayChildren: 0.2 },
            },
          }}
          viewBox="0 0 24 24"
          width={size}
          xmlns="http://www.w3.org/2000/svg"
          aria-hidden="true"
        >
          <motion.path animate={pathControls} custom={3} d="m5 8 6 6" variants={PATH_VARIANTS} />
          <motion.path
            animate={pathControls}
            custom={2}
            d="m4 14 6-6 3-3"
            variants={PATH_VARIANTS}
          />
          <motion.path animate={pathControls} custom={1} d="M2 5h12" variants={PATH_VARIANTS} />
          <motion.path animate={pathControls} custom={0} d="M7 2h1" variants={PATH_VARIANTS} />
          <motion.path
            animate={pathControls}
            custom={3}
            d="m22 22-5-10-5 10"
            variants={PATH_VARIANTS}
          />
          <motion.path animate={pathControls} custom={3} d="M14 18h6" variants={PATH_VARIANTS} />
        </motion.svg>
      </div>
    );
  }
);

LanguagesIcon.displayName = "LanguagesIcon";

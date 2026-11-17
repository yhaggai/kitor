import type { StorybookConfig } from "@storybook/react-vite";

const config: StorybookConfig = {
  stories: ["../src/**/*.stories.@(ts|tsx)"],
  addons: ["@storybook/addon-docs", "@storybook/addon-a11y"],
  framework: "@storybook/react-vite",
  staticDirs: ["../public"],
  viteFinal(config) {
    return {
      ...config,
      build: {
        ...config.build,
        // Storybook copies staticDirs into the build output.
        copyPublicDir: false,
        // The preview includes Storybook's docs renderer and accessibility tooling.
        chunkSizeWarningLimit: 1500,
        rollupOptions: {
          ...config.build?.rollupOptions,
          onwarn(warning, defaultHandler) {
            // The preview renders in the browser, so Motion's React Server Components directives are inert.
            if (
              warning.code === "MODULE_LEVEL_DIRECTIVE" &&
              warning.id?.includes("/node_modules/framer-motion/") &&
              warning.message.includes('"use client"')
            ) {
              return;
            }
            const onwarn = config.build?.rollupOptions?.onwarn ?? defaultHandler;
            onwarn(warning, defaultHandler);
          },
        },
      },
    };
  },
};
export default config;

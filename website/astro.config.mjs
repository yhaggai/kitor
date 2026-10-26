import { defineConfig } from "astro/config";
import react from "@astrojs/react";
import { fileURLToPath } from "node:url";
export default defineConfig({
  integrations: [react()],
  vite: {
    resolve: {
      alias: {
        "@/lib/utils": fileURLToPath(
          new URL("./src/lib/utils.ts", import.meta.url),
        ),
      },
      dedupe: ["react", "react-dom"],
    },
  },
});

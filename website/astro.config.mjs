import { defineConfig } from "astro/config";
import react from "@astrojs/react";
import { fileURLToPath } from "node:url";
export default defineConfig({
  site: process.env.KITOR_SITE_URL || "https://kitor-hebrew-adventures.gigman.chatgpt.site",
  trailingSlash: "always",
  integrations: [react()],
  vite: {
    resolve: {
      alias: {
        "@kitor/ui": fileURLToPath(new URL("./src/lib/box-translation.ts", import.meta.url)),
        "@/lib/utils": fileURLToPath(
          new URL("./src/lib/utils.ts", import.meta.url),
        ),
      },
      dedupe: ["react", "react-dom"],
    },
  },
});

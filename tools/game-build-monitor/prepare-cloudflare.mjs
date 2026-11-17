import { spawnSync } from "node:child_process";
import { cp, mkdir, rm } from "node:fs/promises";
import { fileURLToPath } from "node:url";

const website = new URL("../../website/", import.meta.url);
const staging = new URL(".astro/monitor-cloudflare-assets/", website);
const dist = new URL("dist/", website);
const build = spawnSync("npm", ["run", "build"], {
  cwd: fileURLToPath(website),
  stdio: "inherit",
  env: {
    ...process.env,
    PUBLIC_GAME_BUILD_SITE_ORIGIN: process.env.KITOR_SITE_URL
      || "https://kitor-hebrew-adventures.gigman.chatgpt.site",
    PUBLIC_GAME_BUILD_SNAPSHOT_URL: "/game-builds.json",
  },
});
if (build.error) throw build.error;
if (build.status !== 0) process.exit(build.status ?? 1);

await rm(staging, { force: true, recursive: true });
await mkdir(staging, { recursive: true });
await Promise.all([
  cp(new URL("public/", website), staging, { recursive: true }),
  cp(new URL("_astro/", dist), new URL("_astro/", staging), { recursive: true }),
  cp(new URL("build-monitor/index.html", dist), new URL("index.html", staging)),
]);

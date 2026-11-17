import { mkdir, readFile, writeFile } from "node:fs/promises";
import { monitorTargets } from "./scan.mjs";
import { patchFiles } from "./patch-files.mjs";

const registry = JSON.parse(await readFile(new URL("../../kitor/patches/registry.json", import.meta.url), "utf8"));
const directory = new URL("../../website/.astro/", import.meta.url);
const plans = [];
for (const row of monitorTargets(registry.games)) {
  if (row.store !== "gog") continue;
  for (const baseline of row.supported) {
    if (!baseline.build_id) continue;
    try {
      const files = await patchFiles(row, baseline);
      plans.push({
        gameId: row.gameId,
        platform: baseline.platform,
        language: baseline.language,
        patchVersion: baseline.patchVersion,
        files,
      });
    } catch (error) {
      console.warn(`GOG patch plan unavailable for ${row.gameId}: ${error.message}`);
    }
  }
}
await mkdir(directory, { recursive: true });
await writeFile(new URL("monitor-gog-plans.json", directory), JSON.stringify(plans));

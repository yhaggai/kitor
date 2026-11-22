import { readFile } from "node:fs/promises";
import { isDeepStrictEqual } from "node:util";
import { pathToFileURL } from "node:url";
import { setTimeout } from "node:timers/promises";
import { monitorTargets } from "./scan.mjs";
import { triggerCheck } from "./trigger-check.mjs";

export function hasCurrentCatalog(snapshot, games) {
  const targets = monitorTargets(games);
  if (!Array.isArray(snapshot.rows) || snapshot.rows.length !== targets.length) return false;
  return targets.every(target => {
    const row = snapshot.rows.find(row => row.key === target.key);
    const supported = row?.supported?.map(({ status, fileCheck, ...baseline }) => baseline);
    return isDeepStrictEqual(supported, target.supported);
  });
}

export async function refreshDashboard(games, token, {
  trigger = triggerCheck, transport = fetch, pause = setTimeout, attempts = 6,
} = {}) {
  for (let attempt = 0; attempt < attempts; attempt++) {
    await trigger("scan", token);
    const response = await transport("https://build.kitor.click/game-builds.json", {
      cache: "no-store", redirect: "error", signal: AbortSignal.timeout(30_000),
    });
    if (!response.ok) throw new Error(`Dashboard snapshot returned HTTP ${response.status}`);
    if (hasCurrentCatalog(await response.json(), games)) {
      console.log("Production dashboard shows the published patch catalog.");
      return;
    }
    // KV writes can take time to reach the Worker location serving the scan.
    if (attempt + 1 < attempts) await pause(20_000);
  }
  throw new Error("Production dashboard still shows an outdated patch catalog");
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const { games } = JSON.parse(await readFile(new URL("../../kitor/patches/registry.json", import.meta.url), "utf8"));
  await refreshDashboard(games, process.env.KITOR_MONITOR_TRIGGER_TOKEN);
}

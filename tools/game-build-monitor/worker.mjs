import registry from "../../kitor/patches/registry.json";
import { monitorTargets, scan } from "./scan.mjs";

import plans from "../../website/.astro/monitor-gog-plans.json";
import { sendPatchAlerts } from "./alerts.mjs";
import { inspectChangedGogBuilds } from "./gog-files.mjs";
import { applyFileChecks, extractFileChecks, fileChecksKey } from "./file-check-overlay.mjs";

const snapshotKey = "latest";
const targets = monitorTargets(registry.games);

function lookupGogPlan(row, baseline) {
  return plans.find((plan) => plan.gameId === row.gameId && plan.platform === baseline.platform
    && plan.language === baseline.language && plan.patchVersion === baseline.patchVersion)?.files;
}

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    if (url.pathname === "/build-monitor" || url.pathname === "/build-monitor/") {
      return Response.redirect(new URL("/", url), 308);
    }
    if (url.pathname !== "/game-builds.json") return env.ASSETS.fetch(request);

    const [snapshot, steamChecks, gogChecks] = await Promise.all([
      env.BUILD_SNAPSHOTS.get(snapshotKey, { type: "json" }),
      env.BUILD_SNAPSHOTS.get(fileChecksKey("steam"), { type: "json" }),
      env.BUILD_SNAPSHOTS.get(fileChecksKey("gog"), { type: "json" }),
    ]);
    return snapshot
      ? Response.json(applyFileChecks(applyFileChecks(snapshot, steamChecks, "steam"), gogChecks, "gog"), { headers: { "cache-control": "no-store" } })
      : Response.json({ error: "Store scan pending" }, { status: 404 });
  },

  async scheduled(event, env) {
    if (event.cron === "33 6 * * *") {
      await sendPatchAlerts(env);
      return;
    }
    if (event.cron === "43 5 * * *") {
      const [snapshot, checks] = await Promise.all([
        env.BUILD_SNAPSHOTS.get(snapshotKey, { type: "json" }),
        env.BUILD_SNAPSHOTS.get(fileChecksKey("gog"), { type: "json" }),
      ]);
      if (!snapshot) throw new Error("Store scan pending");
      const previous = applyFileChecks(structuredClone(snapshot), checks, "gog");
      await inspectChangedGogBuilds(snapshot, previous, lookupGogPlan);
      const next = extractFileChecks(snapshot, "gog");
      if (JSON.stringify(next) !== JSON.stringify(checks ?? [])) {
        await env.BUILD_SNAPSHOTS.put(fileChecksKey("gog"), JSON.stringify(next));
      }
      console.log(`GOG manifest comparisons: ${next.length}.`);
      return;
    }
    const previous = await env.BUILD_SNAPSHOTS.get(snapshotKey, { type: "json" });
    const snapshot = await scan(targets, previous);
    const healthy = snapshot.rows.filter((row) => row.availability === "ok").length;
    if (!healthy) throw new Error("No store returned a public build; retained previous snapshot");
    await env.BUILD_SNAPSHOTS.put(snapshotKey, JSON.stringify(snapshot));
    console.log(`Checked ${snapshot.rows.length} game/store entries; ${healthy} returned a public build.`);
  },
};

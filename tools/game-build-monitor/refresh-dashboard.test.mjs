import test from "node:test";
import assert from "node:assert/strict";
import { monitorTargets } from "./scan.mjs";
import { hasCurrentCatalog, refreshDashboard } from "./refresh-dashboard.mjs";

const games = [{ scummvm_id: "moomin", title: "Moomin", steam_app_id: "4463520", patches: [{
  version: "0.1.3-download1", language: "he", supported_builds: [{ store: "steam", platform: "windows", build_id: "26803369" }],
}] }];
const current = () => ({ rows: monitorTargets(games).map(row => ({ ...row,
  supported: row.supported.map(baseline => ({ ...baseline, status: "match", fileCheck: { status: "unchanged" } })),
})) });

test("verification requires the published version and build, ignoring comparison annotations", () => {
  assert.equal(hasCurrentCatalog(current(), games), true);
  const stale = current();
  stale.rows[0].supported[0].patchVersion = "0.1.1";
  assert.equal(hasCurrentCatalog(stale, games), false);
  const wrongBuild = current();
  wrongBuild.rows[0].supported[0].build_id = "26355427";
  assert.equal(hasCurrentCatalog(wrongBuild, games), false);
  assert.equal(hasCurrentCatalog({ rows: [] }, games), false);
});

test("refresh waits for KV propagation and confirms production rather than just trigger success", async () => {
  let scans = 0, pauses = 0;
  await refreshDashboard(games, "test-token", {
    trigger: async (action, token) => { assert.equal(action, "scan"); assert.equal(token, "test-token"); scans++; },
    transport: async () => Response.json(scans === 1 ? { rows: [] } : current()),
    pause: async () => { pauses++; },
  });
  assert.equal(scans, 2);
  assert.equal(pauses, 1);
});

test("a persistently stale production snapshot fails the refresh", async () => {
  await assert.rejects(refreshDashboard(games, "test-token", {
    trigger: async () => {}, transport: async () => Response.json({ rows: [] }),
    pause: async () => {}, attempts: 2,
  }), /outdated patch catalog/);
});

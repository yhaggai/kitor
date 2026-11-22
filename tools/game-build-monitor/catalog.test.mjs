import test from "node:test";
import assert from "node:assert/strict";
import { loadCatalog, catalogKey } from "./catalog.mjs";
import { monitorTargets } from "./scan.mjs";

const game = (version, build) => ({
  scummvm_id: "moomin", title: "Moomin", steam_app_id: "4463520",
  patches: [{ language: "he", version, supported_builds: [{ store: "steam", platform: "windows", build_id: build }] }],
});

test("a release replaces the deployed patch version and build baseline", async () => {
  const fallback = { games: [game("0.1.1", "26355427")], plans: [] };
  const latest = { games: [game("0.1.3-download1", "26803369")], plans: [{ patchVersion: "0.1.3-download1" }] };
  const kv = { async get(key, options) {
    assert.equal(key, catalogKey);
    assert.deepEqual(options, { type: "json" });
    return latest;
  } };
  const loaded = await loadCatalog(kv, fallback);
  const [row] = monitorTargets(loaded.games);
  assert.equal(row.supported[0].patchVersion, "0.1.3-download1");
  assert.equal(row.supported[0].build_id, "26803369");
  assert.deepEqual(loaded.plans, latest.plans);
});

test("an uninitialized catalog retains the deployment baseline", async () => {
  const fallback = { games: [game("0.1.1", "26355427")], plans: [] };
  assert.equal(await loadCatalog({ get: async () => null }, fallback), fallback);
});

test("malformed or unavailable catalogs fail instead of reverting to stale baselines", async () => {
  for (const catalog of [{}, { games: [], plans: [] }, { games: [game("new", "123")] }]) {
    await assert.rejects(loadCatalog({ get: async () => catalog }, {}), /Invalid published monitor catalog/);
  }
  await assert.rejects(loadCatalog({ get: async () => { throw new Error("KV unavailable"); } }, {}), /KV unavailable/);
});

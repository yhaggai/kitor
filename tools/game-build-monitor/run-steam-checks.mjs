import { execFile } from "node:child_process";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { homedir, tmpdir } from "node:os";
import { join } from "node:path";
import { promisify } from "node:util";
import { fileURLToPath } from "node:url";
import { inspectChangedDepots } from "./steam-files.mjs";
import { extractFileChecks, fileChecksKey } from "./file-check-overlay.mjs";

const run = promisify(execFile);
const url = process.env.KITOR_BUILD_SNAPSHOT_URL ?? "https://build.kitor.click/game-builds.json";
const binary = process.env.KITOR_STEAM_MANIFEST_BIN
  ?? fileURLToPath(new URL("../../kitor/native-steam/target/debug/kitor-steam", import.meta.url));
const cache = process.env.KITOR_STEAM_DEPOT_CACHE
  ?? join(homedir(), "Library/Application Support/Steam/depotcache");

const response = await fetch(url, { signal: AbortSignal.timeout(30_000) });
if (!response.ok) throw new Error(`Dashboard snapshot returned HTTP ${response.status}`);
const snapshot = await response.json();
const previous = structuredClone(snapshot);
await inspectChangedDepots(snapshot, previous, { binary, cache });
const store = "steam";
const checks = extractFileChecks(snapshot, store);
console.log(`Steam manifest results for ${snapshot.checkedAt}: ${checks.length} changed-build comparisons.`);
if (JSON.stringify(checks) !== JSON.stringify(extractFileChecks(previous, store))) {
  const directory = await mkdtemp(join(tmpdir(), "kitor-build-checks-"));
  try {
    const output = join(directory, "checks.json");
    await writeFile(output, JSON.stringify(checks));
    const config = fileURLToPath(new URL("./wrangler.jsonc", import.meta.url));
    const env = { ...process.env };
    delete env.KITOR_STEAM_CLOUD_SESSION;
    const { stdout } = await run(process.env.KITOR_WRANGLER_BIN ?? "wrangler", [
      "kv", "key", "put", fileChecksKey(store), "--binding", "BUILD_SNAPSHOTS",
      "--remote", "--config", config, "--path", output,
    ], { env, timeout: 60_000 });
    console.log(stdout.trim());
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
}

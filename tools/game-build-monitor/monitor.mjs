import { readFile, writeFile } from "node:fs/promises";
import { homedir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

import { monitorTargets, scan } from "./scan.mjs";
import { inspectChangedDepots } from "./steam-files.mjs";
export * from "./scan.mjs";

const registry = new URL("../../kitor/patches/registry.json", import.meta.url);
const readJson = async (path) => JSON.parse(await readFile(path, "utf8"));

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [output, previousPath] = process.argv.slice(2);
  if (!output) throw new Error("Usage: node monitor.mjs <output.json> [previous.json]");
  const [{ games }, previous] = await Promise.all([
    readJson(registry), previousPath ? readJson(previousPath) : null,
  ]);
  const snapshot = await scan(monitorTargets(games), previous);
  const binary = process.env.KITOR_STEAM_MANIFEST_BIN;
  if (binary) {
    const cache = process.env.KITOR_STEAM_DEPOT_CACHE
      ?? (process.platform === "darwin" ? join(homedir(), "Library/Application Support/Steam/depotcache") : undefined);
    await inspectChangedDepots(snapshot, previous, { binary, cache });
  }
  await writeFile(output, JSON.stringify(snapshot, null, 2) + "\n");
  const healthy = snapshot.rows.filter((row) => row.availability === "ok").length;
  console.log(`Checked ${snapshot.rows.length} game/store entries; ${healthy} returned a public build.`);
  if (!healthy) process.exitCode = 1;
}

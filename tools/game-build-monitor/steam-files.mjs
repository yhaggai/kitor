import { execFile } from "node:child_process";
import { promisify } from "node:util";

const run = promisify(execFile);
import { patchFiles } from "./patch-files.mjs";
const validHash = (hash) => /^[a-f0-9]{40}$/.test(hash ?? "");

export function compareFiles(before, after, originals, created = [], root) {
  const anchor = originals[0];
  if (!anchor) return { status: "unavailable", reason: "No original files in patch plan" };

  const roots = root !== undefined ? [root] : Object.keys(before)
    .filter((path) => path === anchor || path.endsWith(`/${anchor}`))
    .map((path) => path.slice(0, -anchor.length));
  const matches = roots.filter((prefix) => originals.every((path) => validHash(before[prefix + path])));
  if (matches.length !== 1) {
    return { status: "unavailable", reason: "Patch files do not map to one baseline depot folder" };
  }
  const [prefix] = matches;
  const changed = originals.filter((path) => before[prefix + path] !== after[prefix + path]);
  const added = created.filter((path) => !Object.hasOwn(before, prefix + path)
    && Object.hasOwn(after, prefix + path));
  if (changed.length || added.length) {
    return { status: "changed", paths: [...changed, ...added] };
  }
  return { status: "unchanged", filesChecked: originals.length };
}

export async function inspectChangedDepots(snapshot, previous, { binary, cache }) {
  const manifests = new Map();
  const fetchManifest = (app, depot, manifest) => {
    const key = `${app}:${depot}:${manifest}`;
    if (!manifests.has(key)) {
      const args = ["manifest-hashes", app, depot, manifest];
      if (cache) args.push(cache);
      manifests.set(key, run(binary, args, { timeout: 90_000, maxBuffer: 16 * 1024 * 1024 })
        .then(({ stdout }) => JSON.parse(stdout)));
    }
    return manifests.get(key);
  };

  for (const row of snapshot.rows) {
    if (row.store !== "steam" || row.availability !== "ok") continue;
    const prior = previous?.rows.find((entry) => entry.gameId === row.gameId && entry.key === row.key);
    for (const baseline of row.supported) {
      if (baseline.status !== "different" || !baseline.depot_id || !baseline.manifest_id) continue;
      const current = row.latest.manifests[baseline.depot_id];
      try {
        const { originals, created, root } = await patchFiles(row, baseline);
        const fingerprint = JSON.stringify([baseline.manifest_id, current, baseline.patchVersion, originals, created, root]);
        const saved = prior?.supported.find(({ platform, depot_id, fileCheck }) =>
          platform === baseline.platform && depot_id === baseline.depot_id
          && fileCheck?.fingerprint === fingerprint && fileCheck.status !== "unavailable");
        if (saved) {
          baseline.fileCheck = saved.fileCheck;
          continue;
        }
        const [before, after] = await Promise.all([
          fetchManifest(row.id, baseline.depot_id, baseline.manifest_id),
          fetchManifest(row.id, baseline.depot_id, current),
        ]);
        baseline.fileCheck = { ...compareFiles(before, after, originals, created, root), fingerprint };
      } catch {
        baseline.fileCheck = { status: "unavailable", reason: "Patch plan or Steam depot manifest unavailable" };
      }
    }
  }
}

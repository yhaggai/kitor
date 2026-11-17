import { inflateSync } from "node:zlib";
import { Buffer } from "node:buffer";

const metadataLimit = 32 * 1024 * 1024;
const english = /^(?:\*|neutral|english|en(?:-[a-z]+)?)$/i;
const checksum = (hash, length) =>
  typeof hash === "string" && hash.length === length && /^[\da-f]+$/i.test(hash)
    ? hash.toLowerCase() : null;

function fileHashes(item) {
  const chunks = item.chunks?.map((chunk) => ({
    size: chunk.size, md5: checksum(chunk.md5, 32),
  }));
  const md5 = checksum(item.md5 ?? item.hash, 32)
    ?? (chunks?.length === 1 ? chunks[0].md5 : null);
  return { sha256: checksum(item.sha256, 64), md5, chunks };
}

function compareContent(before, after) {
  for (const name of ["sha256", "md5"]) {
    if (before[name] && after[name]) return before[name] === after[name] ? "unchanged" : "changed";
  }
  // Equal boundaries let us compare uncompressed chunks directly. Rechunking
  // alone gives no proof that the original file changed.
  if (before.chunks?.length && before.chunks.length === after.chunks?.length
    && before.chunks.every((chunk, i) => Number.isSafeInteger(chunk.size) && chunk.md5
      && chunk.size === after.chunks[i].size && after.chunks[i].md5)) {
    return before.chunks.every((chunk, i) => chunk.md5 === after.chunks[i].md5)
      ? "unchanged" : "changed";
  }
  return "unavailable";
}

const fileKey = (path, os) => {
  const name = path.replaceAll("\\", "/").replace(/^\/+/, "");
  return os === "windows" ? name.toLowerCase() : name;
};

export function compareGogFiles(before, after, originals, created, os) {
  if (!originals.length) return { status: "unavailable", reason: "No original files in patch plan" };
  const changed = [];
  const unknown = [];
  for (const path of originals) {
    const key = fileKey(path, os);
    const source = before.get(key);
    const current = after.get(key);
    if (!source || source.length !== 1 || (current && current.length !== 1)) {
      unknown.push(path);
    } else if (!current) {
      changed.push(path);
    } else {
      const comparison = compareContent(source[0], current[0]);
      if (comparison === "changed") changed.push(path);
      if (comparison === "unavailable") unknown.push(path);
    }
  }
  for (const path of created) {
    const key = fileKey(path, os);
    if (!before.has(key) && after.has(key)) changed.push(path);
  }
  if (changed.length) return { status: "changed", paths: changed };
  if (unknown.length) {
    return { status: "unavailable", reason: "No comparable GOG manifest hashes for all patch inputs" };
  }
  return { status: "unchanged", filesChecked: originals.length };
}

async function readMetadata(url, fetcher) {
  const response = await fetcher(url, { signal: AbortSignal.timeout(20_000), redirect: "error" });
  if (!response.ok || !response.body) throw new Error(`GOG metadata returned HTTP ${response.status}`);
  const chunks = [];
  let length = 0;
  for await (const chunk of response.body) {
    length += chunk.length;
    if (length > metadataLimit) throw new Error("GOG manifest exceeds size limit");
    chunks.push(chunk);
  }
  const body = Buffer.concat(chunks);
  return JSON.parse(body[0] === 0x78 ? inflateSync(body, { maxOutputLength: metadataLimit }) : body);
}

function manifestUrl(build, id, os) {
  const url = new URL(build.link);
  const validPath = build.generation === 2
    ? /^\/content-system\/v2\/meta\/[a-f\d]{2}\/[a-f\d]{2}\/[a-f\d]{32}$/.test(url.pathname)
    : url.pathname === `/content-system/v1/manifests/${id}/${os}/${build.legacy_build_id}/repository.json`;
  if (url.protocol !== "https:" || url.username || url.password || url.port
    || !["gog-cdn-fastly.gog.com", "gog-cdn.gcdn.co", "cdn.gog.com"].includes(url.hostname) || !validPath) {
    throw new Error("Unexpected GOG manifest URL");
  }
  return url;
}

async function depotItems(build, id, os, fetcher) {
  const url = manifestUrl(build, id, os);
  const root = await readMetadata(url, fetcher);
  const v2 = build.generation === 2;
  if (v2 && (String(root.baseProductId) !== id || String(root.buildId) !== String(build.build_id))) {
    throw new Error("GOG manifest identity mismatch");
  }
  const depots = (v2 ? root.depots : root.product.depots).filter((depot) =>
    depot.languages?.some((language) => english.test(language))
    && (v2 ? String(depot.productId) === id : depot.gameIDs?.some((game) => String(game) === id)));
  if (!depots.length) throw new Error("No GOG base-game English depots");
  const items = [];
  for (const { manifest } of depots) {
    const pattern = v2 ? /^[a-f\d]{32}$/ : /^[a-f\d-]+\.json$/;
    if (!pattern.test(manifest)) throw new Error("Invalid GOG depot manifest ID");
    const path = v2 ? `/content-system/v2/meta/${manifest.slice(0, 2)}/${manifest.slice(2, 4)}/${manifest}` : manifest;
    const metadata = await readMetadata(new URL(path, url), fetcher);
    const files = v2 ? metadata.depot.items : metadata.depot.files;
    items.push(...files.filter((item) =>
      (v2 ? ["DepotFile", "DepotLink"].includes(item.type) : Boolean(item.path))
      && !item.flags?.includes("support")));
  }
  return items;
}

export async function fetchGogFiles(id, os, buildId, fetcher = fetch, listBuilds) {
  const builds = listBuilds ?? await readMetadata(
    `https://content-system.gog.com/products/${id}/os/${os}/builds?generation=2`, fetcher,
  );
  const build = builds.items.find((item) => String(item.build_id) === buildId
    && [1, 2].includes(item.generation));
  if (!build) throw new Error("GOG no longer lists the requested build manifest");
  const index = new Map();
  for (const item of await depotItems(build, id, os, fetcher)) {
    const key = fileKey(item.path, os);
    const entry = index.get(key) ?? [];
    entry.push(fileHashes(item));
    index.set(key, entry);
  }
  return index;
}

export async function inspectChangedGogBuilds(snapshot, previous, lookupPlan, fetcher = fetch) {
  for (const row of snapshot.rows) {
    if (row.store !== "gog" || row.availability !== "ok") continue;
    const prior = previous?.rows.find((entry) => entry.gameId === row.gameId && entry.key === row.key);
    for (const baseline of row.supported) {
      if (baseline.status !== "different" || !baseline.build_id) continue;
      try {
        const { originals, created } = await lookupPlan(row, baseline);
        const current = row.latest.buildId;
        const fingerprint = JSON.stringify([baseline.build_id, current, baseline.patchVersion, originals, created]);
        const saved = prior?.supported.find(({ platform, fileCheck }) =>
          platform === baseline.platform && fileCheck?.fingerprint === fingerprint
          && fileCheck.status !== "unavailable");
        if (saved) {
          baseline.fileCheck = saved.fileCheck;
          continue;
        }
        const list = await readMetadata(
          `https://content-system.gog.com/products/${row.id}/os/${row.os}/builds?generation=2`, fetcher,
        );
        const [before, after] = await Promise.all([
          fetchGogFiles(row.id, row.os, baseline.build_id, fetcher, list),
          fetchGogFiles(row.id, row.os, current, fetcher, list),
        ]);
        baseline.fileCheck = { ...compareGogFiles(before, after, originals, created, row.os), fingerprint };
      } catch {
        baseline.fileCheck = { status: "unavailable", reason: "Patch plan or GOG build manifest unavailable" };
      }
    }
  }
}

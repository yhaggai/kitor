const storeId = (value) => (/^[1-9]\d*$/.test(String(value)) ? String(value) : null);

export function monitorTargets(games) {
  return games
    .filter((game) => game.patches?.length)
    .flatMap((game) => {
      const targets = [];
      const steam = storeId(game.steam_app_id);
      const gog = storeId(game.gog?.id);
      if (steam) targets.push({ key: `steam:${steam}`, store: "steam", id: steam });
      if (gog) {
        for (const [flag, os] of [
          ["windows", "windows"],
          ["osx", "osx"],
          ["linux", "linux"],
        ]) {
          if (game.platforms?.[flag]) {
            targets.push({ key: `gog:${gog}:${os}`, store: "gog", id: gog, os });
          }
        }
      }
      return targets.map((target) => ({
        ...target,
        gameId: game.scummvm_id,
        title: game.title,
        supported: game.patches.flatMap((patch) =>
          (patch.supported_builds ?? [])
            .filter(
              (baseline) =>
                baseline.store === target.store && (!target.os || baseline.platform === target.os)
            )
            .map((baseline) => ({
              ...baseline,
              language: patch.language,
              patchVersion: patch.version,
            }))
        ),
        verifiedMismatches: game.patches.flatMap((patch) =>
          (patch.verified_mismatches ?? [])
            .filter(
              (check) =>
                check.store === target.store && (!target.os || check.platform === target.os)
            )
            .map((check) => ({ ...check, language: patch.language, patchVersion: patch.version }))
        ),
      }));
    });
}

function releaseDate(timestamp) {
  return Number.isFinite(timestamp) && timestamp > 0
    ? new Date(timestamp).toISOString().slice(0, 10)
    : null;
}

export function steamBuild(payload, id) {
  const app = payload.data?.[id];
  const depots = app?.depots;
  const branch = depots?.branches?.public;
  const buildId = storeId(branch?.buildid);
  if (!buildId) throw new Error("Steam public build ID unavailable");

  const manifests = {};
  for (const [depot, content] of Object.entries(depots)) {
    const manifestId = storeId(content?.manifests?.public?.gid);
    if (storeId(depot) && manifestId) manifests[depot] = manifestId;
  }
  const branchUpdatedAt = Number(branch.timeupdated) * 1000;
  const appReleasedAt = Number(app.common?.steam_release_date) * 1000;
  const releasedOn = releaseDate(Math.max(branchUpdatedAt, appReleasedAt || 0));
  return { buildId, manifests, releasedOn };
}

export function gogBuild(payload) {
  if (!Array.isArray(payload.items)) throw new Error("Invalid GOG build response");

  const builds = payload.items
    .filter((item) => item.public === true && !item.branch)
    .map((item) => ({ item, published: Date.parse(item.date_published) }))
    .filter(({ published }) => Number.isFinite(published))
    .sort((a, b) => b.published - a.published);
  const latest = builds[0]?.item;
  if (!latest && !payload.items.some((item) => item.public === true && !item.branch)) return null;

  const buildId = storeId(latest?.build_id);
  if (!buildId) throw new Error("GOG public build ID or publication date unavailable");
  return {
    buildId,
    version: latest.version_name || null,
    releasedOn: releaseDate(builds[0].published),
  };
}

export function checkResult(previous, latest, checkedAt) {
  const lastKnown = previous?.latest ?? null;
  const changed =
    latest &&
    lastKnown &&
    (latest.buildId !== lastKnown.buildId ||
      Object.entries(latest.manifests ?? {}).some(
        ([depot, manifest]) =>
          lastKnown.manifests?.[depot] && lastKnown.manifests[depot] !== manifest
      ));
  return {
    latest: latest ?? lastKnown,
    availability: latest ? "ok" : "unavailable",
    lastSuccessAt: latest ? checkedAt : (previous?.lastSuccessAt ?? null),
    change: changed
      ? { previousBuildId: lastKnown.buildId, buildId: latest.buildId, detectedAt: checkedAt }
      : (previous?.change ?? null),
  };
}

export function compareBaseline(baseline, latest) {
  if (!latest) return "unavailable";
  const current = baseline.depot_id ? latest.manifests?.[baseline.depot_id] : latest.buildId;
  if (!current) return "unavailable";
  return current === (baseline.manifest_id ?? baseline.build_id) ? "match" : "different";
}

async function fetchJson(url) {
  const response = await fetch(url, { signal: AbortSignal.timeout(15_000) });
  if (!response.ok) throw new Error(`HTTP ${response.status}`);
  return response.json();
}

async function fetchBuild({ store, id, os }) {
  if (store === "steam") {
    return steamBuild(await fetchJson(`https://api.steamcmd.net/v1/info/${id}`), id);
  }
  return gogBuild(
    await fetchJson(`https://content-system.gog.com/products/${id}/os/${os}/builds?generation=2`)
  );
}

export async function scan(targets, previous, lookup = fetchBuild) {
  const checkedAt = new Date().toISOString();
  const sources = new Map(targets.map((target) => [target.key, target]));
  const prior = new Map(previous?.rows.map((row) => [row.key, row]) ?? []);
  const keys = [...sources.keys()];
  const checks = new Map();

  for (let offset = 0; offset < keys.length; offset += 4) {
    await Promise.all(
      keys.slice(offset, offset + 4).map(async (key) => {
        try {
          checks.set(key, checkResult(prior.get(key), await lookup(sources.get(key)), checkedAt));
        } catch (error) {
          console.warn(`${key}: ${error.message}`);
          checks.set(key, {
            ...checkResult(prior.get(key), null, checkedAt),
            availability: "error",
          });
        }
      })
    );
  }

  const rows = targets.map((target) => {
    const { verifiedMismatches = [], ...row } = target;
    const check = checks.get(row.key);
    const supported = target.supported.map((baseline) => ({
      ...baseline,
      status: compareBaseline(baseline, check.latest),
    }));
    for (const { paths, checked_manifest_id, ...comparison } of verifiedMismatches) {
      const matchesManifest =
        check.availability === "ok" &&
        check.latest?.manifests?.[comparison.depot_id] === checked_manifest_id;
      supported.push({
        ...comparison,
        status: "unavailable",
        ...(matchesManifest && { fileCheck: { status: "changed", paths } }),
      });
    }
    return { ...row, ...check, supported };
  });
  return { checkedAt, rows };
}

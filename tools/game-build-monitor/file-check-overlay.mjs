export const fileChecksKey = (store) => `${store}-file-checks`;

function identity(row, baseline) {
  const patch = [row.gameId, row.key, baseline.platform, baseline.language, baseline.patchVersion];
  return row.store === "steam"
    ? [...patch, baseline.depot_id, baseline.manifest_id, row.latest?.manifests?.[baseline.depot_id]]
    : [...patch, baseline.build_id, row.latest?.buildId];
}

export function extractFileChecks(snapshot, store) {
  return snapshot.rows.filter((row) => row.store === store).flatMap((row) => row.supported
    .filter((baseline) => baseline.fileCheck)
    .map((baseline) => ({ identity: identity(row, baseline), check: baseline.fileCheck })));
}

export function applyFileChecks(snapshot, checks, store) {
  const index = new Map((checks ?? []).map(({ identity, check }) => [JSON.stringify(identity), check]));
  for (const row of snapshot.rows) {
    for (const baseline of row.supported) {
      if (row.store !== store || row.availability !== "ok" || baseline.status !== "different") continue;
      const check = index.get(JSON.stringify(identity(row, baseline)));
      if (check) baseline.fileCheck = check;
    }
  }
  return snapshot;
}

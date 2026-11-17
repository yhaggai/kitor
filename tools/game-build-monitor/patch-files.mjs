import { readFile } from "node:fs/promises";

const workspace = new URL("../../patches-workspace/", import.meta.url);

export async function patchFiles(row, baseline) {
  // The Classic patch consumes data extracted from the Remastered archive.
  if (row.store === "steam" && row.gameId === "tentacle") {
    return {
      originals: [baseline.platform === "osx" ? "Dott.app/Contents/Resources/tenta.cle" : "tenta.cle"],
      created: [],
      root: "",
    };
  }
  const folder = new URL(`${row.gameId}/${baseline.language}/`, workspace);
  for (const name of ["patch.json", `patch.${baseline.platform}.candidate.json`]) {
    let plan;
    try {
      plan = JSON.parse(await readFile(new URL(name, folder), "utf8"));
    } catch (error) {
      if (error.code === "ENOENT") continue;
      throw error;
    }
    if (!Object.hasOwn(plan.platforms ?? {}, baseline.platform)) continue;
    const sourceVersion = baseline.patchVersion.replace(/-download\d+$/, "");
    if (plan.version !== baseline.patchVersion && plan.version !== sourceVersion) continue;
    const files = plan.platforms[baseline.platform].files ?? plan.files ?? [];
    if (files.some(({ operation, original_sha256 }) => operation !== "create" && !original_sha256)) {
      throw new Error("Patch plan has inputs without recorded original hashes");
    }
    return {
      originals: files.filter((file) => file.original_sha256).map(({ path }) => path),
      created: files.filter(({ operation }) => operation === "create").map(({ path }) => path),
      root: row.store === "steam" && row.gameId === "sq1" ? "2016_SpaceQuestCollection/sq1/" : undefined,
    };
  }
  throw new Error("Matching patch plan unavailable");
}

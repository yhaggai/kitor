import { createHash } from "node:crypto";
import { pathToFileURL } from "node:url";
import { extractFileChecks } from "./file-check-overlay.mjs";

export function issueCandidates(snapshot) {
  const candidates = [];
  for (const row of snapshot.rows) {
    if (row.availability !== "ok" || !["steam", "gog"].includes(row.store)) continue;
    for (const { identity, check } of extractFileChecks({ rows: [{ ...row, supported: row.supported.filter(b => b.status === "different") }] }, row.store)) {
      if (check.status !== "changed" || !check.paths?.length) continue;
      const [, , platform, language, patchVersion] = identity;
      const id = createHash("sha256").update(JSON.stringify(identity)).digest("hex");
      const marker = `<!-- kitor-build-update:${id} -->`;
      const store = row.store === "steam" ? "Steam" : "GOG";
      const current = row.latest.buildId;
      candidates.push({
        marker,
        title: `[Patch update] ${row.title}: ${store} ${platform} build ${current}`,
        body: [
          marker,
          `The build monitor confirmed changes to files used by the **${row.title}** patch.`,
          `- Store/platform: ${store} / ${platform}`,
          `- Translation/patch: ${language} / ${patchVersion}`,
          `- Supported build: ${row.supported.find(b => b.platform === platform && b.language === language && b.patchVersion === patchVersion && (!b.depot_id || b.depot_id === identity[5]))?.build_id ?? "not recorded"}`,
          `- Current build: ${current}`,
          `- ${row.store === "steam" ? "Depot manifests" : "Builds"}: ${identity.at(-2)} → ${identity.at(-1)}`,
          "",
          "Changed or missing patch inputs:",
          ...check.paths.map(path => `- \`${path}\``),
          "",
          "Verify and update the patch against this build before marking it supported. This file comparison does not establish gameplay compatibility.",
          "",
          "[Build monitor](https://build.kitor.click/)",
          "",
          "Created automatically. Repeated checks reuse this comparison's issue; closing it acknowledges the comparison and prevents it from being recreated.",
        ].join("\n"),
      });
    }
  }
  return candidates;
}

export async function reportBuildIssues(snapshot, repository, github) {
  if (!/^[A-Za-z0-9][\w.-]*\/[A-Za-z0-9][\w.-]*$/.test(repository ?? "")) throw new Error("Invalid GitHub repository");
  const candidates = issueCandidates(snapshot);
  if (!candidates.length) return { created: 0, updated: 0 };
  const issues = [];
  for (let page = 1; ; page++) {
    const batch = await github("GET", `/repos/${repository}/issues?state=all&per_page=100&page=${page}`);
    if (!Array.isArray(batch)) throw new Error("Invalid GitHub issue response");
    issues.push(...batch.filter(issue => !issue.pull_request));
    if (batch.length < 100) break;
  }
  let created = 0, updated = 0;
  for (const { marker, title, body } of candidates) {
    const existing = issues.find(issue => issue.body?.includes(marker));
    if (existing) {
      if (existing.state === "open" && (existing.body !== body || existing.title !== title)) {
        await github("PATCH", `/repos/${repository}/issues/${existing.number}`, { title, body });
        updated++;
      }
      continue;
    }
    const issue = await github("POST", `/repos/${repository}/issues`, { title, body });
    issues.push({ ...issue, title, body });
    created++;
  }
  return { created, updated };
}

async function main() {
  const token = process.env.GH_TOKEN;
  if (!token) throw new Error("GitHub issue token is not configured");
  const response = await fetch("https://build.kitor.click/game-builds.json", {
    redirect: "error", signal: AbortSignal.timeout(30_000),
  });
  if (!response.ok) throw new Error(`Monitor snapshot returned HTTP ${response.status}`);
  const snapshot = await response.json();
  const github = async (method, path, body) => {
    const response = await fetch(`https://api.github.com${path}`, {
      method, redirect: "error", signal: AbortSignal.timeout(30_000),
      headers: { Authorization: `Bearer ${token}`, Accept: "application/vnd.github+json", "X-GitHub-Api-Version": "2022-11-28", "Content-Type": "application/json" },
      ...(body ? { body: JSON.stringify(body) } : {}),
    });
    if (!response.ok) throw new Error(`GitHub issue API returned HTTP ${response.status}`);
    return response.json();
  };
  const result = await reportBuildIssues(snapshot, process.env.GITHUB_REPOSITORY, github);
  console.log(`Patch update issues: ${result.created} created, ${result.updated} updated.`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) await main();

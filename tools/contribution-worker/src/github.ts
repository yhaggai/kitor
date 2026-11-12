import { importPKCS8, SignJWT } from "jose";
import { apply, type Correction, manifestChecksum } from "./csv";
import { type Contributor, type Env, HttpError, hash, now } from "./shared";

const REPO = "yhaggai/kitor";
const CSV = "patches-workspace/drifter/he/payload/translation.csv",
  MANIFEST = "patches-workspace/drifter/he/patch.json";
class GitHubError extends HttpError {
  constructor(readonly upstreamStatus: number) {
    super(502, `GitHub returned ${upstreamStatus}; retry shortly`);
  }
}

export async function github(
  path: string,
  token: string,
  body?: unknown,
  raw = false,
  method = body ? "POST" : "GET"
): Promise<any> {
  const response = await fetch(`https://api.github.com/${path}`, {
    method,
    redirect: "manual",
    headers: {
      Authorization: `Bearer ${token}`,
      "User-Agent": "Kitor-contributions",
      Accept: raw ? "application/vnd.github.raw+json" : "application/vnd.github+json",
      "X-GitHub-Api-Version": "2022-11-28",
      ...(body ? { "Content-Type": "application/json" } : {}),
    },
    body: body ? JSON.stringify(body) : undefined,
    signal: AbortSignal.timeout(20000),
  });
  if (!response.ok) throw new GitHubError(response.status);
  const reader = response.body!.getReader();
  let text = "",
    size = 0;
  const decoder = new TextDecoder();
  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    size += value.length;
    if (size > 8_000_000) {
      await reader.cancel();
      throw new HttpError(502, "GitHub response too large");
    }
    text += decoder.decode(value, { stream: true });
  }
  text += decoder.decode();
  return raw ? text : JSON.parse(text);
}
export async function installation(env: Env) {
  if (!env.GITHUB_PRIVATE_KEY) throw new HttpError(503, "Contribution bot is not configured");
  const key = await importPKCS8(env.GITHUB_PRIVATE_KEY.replaceAll("\\n", "\n"), "RS256");
  const jwt = await new SignJWT({})
    .setProtectedHeader({ alg: "RS256" })
    .setIssuer(env.GITHUB_APP_ID)
    .setIssuedAt(now() - 60)
    .setExpirationTime(now() + 540)
    .sign(key);
  const i = await github(`repos/${REPO}/installation`, jwt);
  if (String(i.app_id) !== env.GITHUB_APP_ID)
    throw new HttpError(502, "Unexpected bot installation");
  const result = await github(`app/installations/${i.id}/access_tokens`, jwt, {
    repositories: ["kitor"],
    permissions: { contents: "write", pull_requests: "write" },
  });
  return result.token as string;
}
export const source = (path: string, sha: string, token: string) =>
  github(`repos/${REPO}/contents/${path}?ref=${encodeURIComponent(sha)}`, token, undefined, true);
export async function canonical(base: string, token: string) {
  const repo = await github(`repos/${REPO}`, token);
  const branch = repo.default_branch as string;
  const compare = await github(
    `repos/${REPO}/compare/${base}...${encodeURIComponent(branch)}`,
    token
  );
  if (!["ahead", "identical"].includes(compare.status))
    throw new HttpError(409, "Draft base is no longer on upstream; refresh your draft");
  return { branch, csv: await source(CSV, base, token) };
}
export function attribution(user: Contributor) {
  const name = user.name
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/[\\`*_{}[\]()#+.!|~-]/g, "\\$&")
    .replace(/@/g, "&#64;");
  return `Contributor: **${name}**\nKitor contributor ID: \`${user.id}\`\nSigned in with Google. Email is kept private.`;
}
export async function openPR(
  input: Correction,
  csv: string,
  branch: string,
  id: string,
  user: Contributor,
  token: string
) {
  const head = `kitor-contributions/${id}`;
  const endpoint = `repos/${REPO}`;
  const existing = await github(
    `${endpoint}/pulls?state=all&head=${encodeURIComponent(`yhaggai:${head}`)}&base=${encodeURIComponent(branch)}`,
    token
  );
  if (existing.length) {
    if (existing[0].state === "closed")
      throw new HttpError(409, "This PR closed during submission. Retry to create a new PR.");
    await labelContribution(existing[0], token);
    return { url: existing[0].html_url, existing: true };
  }
  const manifest = manifestChecksum(await source(MANIFEST, input.base_sha, token), input.digest);
  const refs = await github(`${endpoint}/git/matching-refs/heads/${head}`, token);
  const ref = refs.find((r: any) => r.ref === `refs/heads/${head}`);
  if (ref) {
    if (
      (await source(CSV, ref.object.sha, token)) !== csv ||
      (await source(MANIFEST, ref.object.sha, token)) !== manifest
    )
      throw new HttpError(409, "Submission branch differs from expected corrections");
  } else {
    const commit = await correctionCommit(input.base_sha, csv, manifest, token);
    await github(`${endpoint}/git/refs`, token, { ref: `refs/heads/${head}`, sha: commit });
  }
  const pr = await github(`${endpoint}/pulls`, token, {
    head,
    base: branch,
    draft: true,
    title: `[POC — do not merge] Drifter: ${Object.keys(input.edits).length} correction(s) from Kitor`,
    body: `${attribution(user)}\n\nSubmitted by the Kitor bot. Base: \`${input.base_sha}\`.\n\n**POC: do not merge or publish.** CSV corrections and checksum only; no release version bump or gameplay validation. Hebrew uses Drifter’s stored display order.`,
  });
  await labelContribution(pr, token);
  return { url: pr.html_url, existing: false };
}

type OpenPull = {
  number: number;
  labels?: { name: string }[];
  html_url: string;
  state: string;
  head: { ref: string; sha: string; repo?: { full_name: string } };
  base: { ref: string; repo?: { full_name: string } };
};
const endpoint = `repos/${REPO}`;

const CONTRIBUTION_LABEL = "translation contribution";

async function labelContribution(pr: Pick<OpenPull, "number" | "labels">, token: string) {
  if (
    !Number.isSafeInteger(pr.number) ||
    pr.labels?.some((label) => label.name === CONTRIBUTION_LABEL)
  )
    return;
  const labelPath = `${endpoint}/labels/${encodeURIComponent(CONTRIBUTION_LABEL)}`;
  try {
    try {
      await github(labelPath, token);
    } catch (error) {
      if (!(error instanceof GitHubError) || error.upstreamStatus !== 404) throw error;
      try {
        await github(`${endpoint}/labels`, token, {
          name: CONTRIBUTION_LABEL,
          color: "0e8a16",
          description: "Hebrew translation contribution submitted through Kitor",
        });
      } catch (createError) {
        if (!(createError instanceof GitHubError) || createError.upstreamStatus !== 422)
          throw createError;
        // Another submission may have created the label concurrently.
        await github(labelPath, token);
      }
    }
    await github(`${endpoint}/issues/${pr.number}/labels`, token, { labels: [CONTRIBUTION_LABEL] });
  } catch (error) {
    // Labeling is metadata. The contributor still needs the URL of their saved PR.
    console.warn("Contribution PR label could not be applied", {
      number: pr.number,
      status: error instanceof GitHubError ? error.upstreamStatus : undefined,
    });
  }
}

export async function findOpenPR(env: Env, contributor: string, branch: string, token: string) {
  const owned = await env.DB.prepare(
    "SELECT DISTINCT url FROM submissions WHERE contributor_id=? AND status='complete' AND url IS NOT NULL"
  )
    .bind(contributor)
    .all<{ url: string }>();
  if (!owned.results.length) return null;
  const urls = new Set(owned.results.map((row) => row.url));
  for (let page = 1; ; page++) {
    const pulls: OpenPull[] = await github(
      `${endpoint}/pulls?state=open&base=${encodeURIComponent(branch)}&sort=created&direction=desc&per_page=100&page=${page}`,
      token
    );
    const match = pulls.find(
      (pr) =>
        urls.has(pr.html_url) &&
        pr.state === "open" &&
        pr.base.ref === branch &&
        pr.base.repo?.full_name === REPO &&
        pr.head.repo?.full_name === REPO &&
        /^kitor-contributions\/[a-f0-9]{64}$/.test(pr.head.ref)
    );
    if (match) return match;
    if (pulls.length < 100) return null;
  }
}

async function correctionCommit(
  parent: string,
  csv: string,
  manifest: string,
  token: string
): Promise<string> {
  const base = await github(`${endpoint}/git/commits/${parent}`, token);
  const tree = [];
  for (const [path, content] of [
    [CSV, csv],
    [MANIFEST, manifest],
  ]) {
    const blob = await github(`${endpoint}/git/blobs`, token, { content, encoding: "utf-8" });
    tree.push({ path, mode: "100644", type: "blob", sha: blob.sha });
  }
  const createdTree = await github(`${endpoint}/git/trees`, token, {
    base_tree: base.tree.sha,
    tree,
  });
  const commit = await github(`${endpoint}/git/commits`, token, {
    message: "POC: Drifter translation corrections from Kitor",
    tree: createdTree.sha,
    parents: [parent],
  });
  return commit.sha;
}

export async function appendPR(pr: OpenPull, input: Correction, token: string) {
  const path = `${endpoint}/pulls/${pr.number}`;
  const current: OpenPull = await github(path, token);
  if (
    current.state !== "open" ||
    current.head.sha !== pr.head.sha ||
    current.head.ref !== pr.head.ref ||
    current.head.repo?.full_name !== REPO ||
    current.base.ref !== pr.base.ref ||
    current.base.repo?.full_name !== REPO
  )
    throw new HttpError(409, "The PR changed during submission. Retry shortly.");
  const previous = await source(CSV, current.head.sha, token);
  const csv = await apply(previous, input.edits);
  if (csv !== previous) {
    const manifest = manifestChecksum(
      await source(MANIFEST, current.head.sha, token),
      await hash(csv)
    );
    const commit = await correctionCommit(current.head.sha, csv, manifest, token);
    const latest: OpenPull = await github(path, token);
    if (
      latest.state !== "open" ||
      latest.head.sha !== current.head.sha ||
      latest.base.ref !== current.base.ref ||
      latest.head.ref !== current.head.ref
    )
      throw new HttpError(409, "The PR changed during submission. Retry shortly.");
    await github(
      `${endpoint}/git/refs/heads/${current.head.ref}`,
      token,
      { sha: commit, force: false },
      false,
      "PATCH"
    );
    const updated = await github(path, token);
    if (updated.state !== "open" && (!updated.merged || updated.head.sha !== commit))
      throw new HttpError(409, "The PR closed during submission. Retry to create a new PR.");
  }
  await labelContribution(current, token);
  return { url: pr.html_url, existing: true, updated: csv !== previous };
}

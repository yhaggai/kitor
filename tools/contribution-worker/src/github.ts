import { importPKCS8, SignJWT } from "jose";
import { type Correction, manifestChecksum } from "./csv";
import { type Contributor, type Env, HttpError, now } from "./shared";

const REPO = "yhaggai/kitor";
const CSV = "patches-workspace/drifter/he/payload/translation.csv",
  MANIFEST = "patches-workspace/drifter/he/patch.json";
export async function github(
  path: string,
  token: string,
  body?: unknown,
  raw = false
): Promise<any> {
  const response = await fetch(`https://api.github.com/${path}`, {
    method: body ? "POST" : "GET",
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
  if (!response.ok) throw new HttpError(502, `GitHub returned ${response.status}; retry shortly`);
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
  if (existing.length) return { url: existing[0].html_url, existing: true };
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
    const base = await github(`${endpoint}/git/commits/${input.base_sha}`, token);
    const tree = [];
    for (const [path, content] of [
      [CSV, csv],
      [MANIFEST, manifest],
    ]) {
      const blob = await github(`${endpoint}/git/blobs`, token, { content, encoding: "utf-8" });
      tree.push({ path, mode: "100644", type: "blob", sha: blob.sha });
    }
    const t = await github(`${endpoint}/git/trees`, token, { base_tree: base.tree.sha, tree });
    const commit = await github(`${endpoint}/git/commits`, token, {
      message: "POC: Drifter translation corrections from Kitor",
      tree: t.sha,
      parents: [input.base_sha],
    });
    await github(`${endpoint}/git/refs`, token, { ref: `refs/heads/${head}`, sha: commit.sha });
  }
  const pr = await github(`${endpoint}/pulls`, token, {
    head,
    base: branch,
    draft: true,
    title: `[POC — do not merge] Drifter: ${Object.keys(input.edits).length} correction(s) from Kitor`,
    body: `${attribution(user)}\n\nSubmitted by the Kitor bot. Base: \`${input.base_sha}\`.\n\n**POC: do not merge or publish.** CSV corrections and checksum only; no release version bump or gameplay validation. Hebrew uses Drifter’s stored display order.`,
  });
  return { url: pr.html_url, existing: false };
}

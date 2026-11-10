import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";
import { build } from "esbuild";
import { createLocalJWKSet, exportJWK, generateKeyPair, SignJWT } from "jose";
import { convertV4MiniflareOptions, Miniflare } from "miniflare";
import { identity } from "../src/auth";
import { apply, manifestChecksum, records, validate } from "../src/csv";
import { attribution } from "../src/github";
import { challenge, type Env, hash } from "../src/shared";

const source = await readFile(
  new URL("../../../patches-workspace/drifter/he/payload/translation.csv", import.meta.url),
  "utf8"
);
test("full Drifter source and byte-preserving correction matches desktop", async () => {
  assert.equal(records(source).length - 3, 9799);
  assert.equal(await apply(source, {}), source);
  const row = records(source).find((r) => r.fields[4] === "Ok" && r.fields[3] === "ContinueGame")!;
  const key = await hash(JSON.stringify(row.fields.slice(0, 5)));
  assert.equal(
    await apply(source, { [key]: "רדסב" }),
    source.replace(",1,Room- Title,ContinueGame,Ok,רושיא", ",1,Room- Title,ContinueGame,Ok,רדסב")
  );
  await assert.rejects(apply(source, { missing: "bad" }));
  const value = 'שורה, "אחת"\nשנייה';
  assert.equal(
    records(await apply(source, { [key]: value })).find(
      (r) => r.fields[4] === "Ok" && r.fields[3] === "ContinueGame"
    )!.fields[5],
    value
  );
});
test("reject impersonation fields and preserve manifest formatting", async () => {
  assert.throws(() =>
    validate({
      base_sha: "a".repeat(40),
      digest: "b".repeat(64),
      edits: { ["c".repeat(64)]: "x" },
      contributor: { name: "pretender" },
    })
  );
  const m = await readFile(
    new URL("../../../patches-workspace/drifter/he/patch.json", import.meta.url),
    "utf8"
  );
  const old = JSON.parse(m).files[0].source.sha256;
  assert.equal(manifestChecksum(m, "a".repeat(64)), m.replace(old, "a".repeat(64)));
  assert.ok(
    !attribution({ id: "id", name: "@everyone <script> [link](https://bad)" }).includes("@everyone")
  );
});
test("Google verification requires signed correct audience, issuer, nonce and verified email", async () => {
  const { privateKey, publicKey } = await generateKeyPair("RS256");
  const jwk = await exportJWK(publicKey);
  jwk.kid = "test";
  const local = createLocalJWKSet({ keys: [jwk] });
  const env = { GOOGLE_CLIENT_ID: "client" } as Env;
  const token = (
    claims: any = {},
    aud = "client",
    iss = "https://accounts.google.com",
    exp = "2m"
  ) =>
    new SignJWT({
      nonce: "nonce",
      email: "test@example.com",
      email_verified: true,
      name: "Test Contributor",
      ...claims,
    })
      .setProtectedHeader({ alg: "RS256", kid: "test" })
      .setSubject("google-id")
      .setAudience(aud)
      .setIssuer(iss)
      .setIssuedAt()
      .setExpirationTime(exp)
      .sign(privateKey);
  assert.equal((await identity(await token(), env, "nonce", local)).name, "Test Contributor");
  for (const t of [
    await token({}, "wrong"),
    await token({}, "client", "evil"),
    await token({ email_verified: false }),
    await token({ email: undefined }),
    await token({ email: "invalid" }),
    await token({ nonce: "wrong" }),
    await token({}, "client", "https://accounts.google.com", "-2m"),
  ])
    await assert.rejects(identity(t, env, "nonce", local));
});
test("real D1: protected endpoints, one-time login handoff, cancellation, session revocation, atomic lease", async () => {
  const bundle = await build({
    entryPoints: ["src/index.ts"],
    bundle: true,
    write: false,
    format: "esm",
    platform: "browser",
    target: "es2022",
  });
  const mf = new Miniflare(
    convertV4MiniflareOptions({
      workers: [
        {
          modules: true,
          script: bundle.outputFiles[0].text,
          compatibilityDate: "2026-11-01",
          d1Databases: ["DB"],
          bindings: {
            PUBLIC_URL: "http://127.0.0.1:8787",
            GITHUB_APP_ID: "5666467",
            GOOGLE_CLIENT_ID: "test",
            GOOGLE_CLIENT_SECRET: "test",
          },
        },
      ],
    })
  );
  try {
    const db = await mf.getD1Database("DB");
    const sql =
      (await readFile("migrations/0001_identity.sql", "utf8")) +
      (await readFile("migrations/0002_notifications.sql", "utf8"));
    for (const statement of sql.split(";").filter((s) => s.trim()))
      await db.prepare(statement).run();
    const post = (path: string, b: any, token?: string) =>
      mf.dispatchFetch("http://localhost" + path, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          ...(token ? { Authorization: `Bearer ${token}` } : {}),
        },
        body: JSON.stringify(b),
      });
    assert.equal((await post("/submissions", {})).status, 401);
    const verifier = "a".repeat(64);
    const start = (await (
      await post("/auth/start", { challenge: await challenge(verifier) })
    ).json()) as any;
    assert.equal(
      new URL(start.authorization_url).searchParams.get("code_challenge_method"),
      "S256"
    );
    assert.equal((await post("/auth/poll", { id: start.id, verifier: "wrong" })).status, 401);
    assert.equal(
      ((await (await post("/auth/poll", { id: start.id, verifier })).json()) as any).pending,
      true
    );
    await db
      .prepare("INSERT INTO contributors(id,google_sub,name,created_at) VALUES (?,?,?,?)")
      .bind("test-user", "sub", "Test Contributor", 0)
      .run();
    await db
      .prepare("UPDATE login_attempts SET status='ready',contributor_id='test-user' WHERE id=?")
      .bind(start.id)
      .run();
    const session = (await (await post("/auth/poll", { id: start.id, verifier })).json()) as any;
    assert.equal(session.contributor.name, "Test Contributor");
    assert.ok(!JSON.stringify(session).includes("email"));
    assert.equal((await post("/auth/poll", { id: start.id, verifier })).status, 401);
    const me = () =>
      mf.dispatchFetch("http://localhost/auth/me", {
        headers: { Authorization: `Bearer ${session.token}` },
      });
    assert.equal((await me()).status, 200);
    await db.prepare("UPDATE sessions SET expires_at=0").run();
    assert.equal((await me()).status, 401);
    await db
      .prepare("UPDATE sessions SET expires_at=?")
      .bind(Math.floor(Date.now() / 1000) + 60)
      .run();
    assert.equal((await post("/auth/logout", {}, session.token)).status, 200);
    assert.equal((await me()).status, 401);
    const second = (await (
      await post("/auth/start", { challenge: await challenge(verifier) })
    ).json()) as any;
    await post("/auth/cancel", { id: second.id, verifier });
    assert.equal((await post("/auth/poll", { id: second.id, verifier })).status, 401);
    const lease = () =>
      db
        .prepare(
          "INSERT INTO submissions(id,contributor_id,status,lease_until,url,created_at) VALUES ('same','test-user','pending',9999999999,NULL,0) ON CONFLICT(id) DO UPDATE SET lease_until=9999999999 WHERE submissions.lease_until<1 RETURNING id"
        )
        .first();
    const results = await Promise.all([lease(), lease()]);
    assert.equal(results.filter(Boolean).length, 1);
  } finally {
    await mf.dispose();
  }
});

test("PR uses bot credentials and server-derived contributor attribution", async () => {
  const { openPR } = await import("../src/github");
  const originalFetch = globalThis.fetch;
  const calls: { url: string; body: Record<string, unknown> | undefined }[] = [];
  const manifest = await readFile(
    new URL("../../../patches-workspace/drifter/he/patch.json", import.meta.url),
    "utf8"
  );
  globalThis.fetch = async (url, options) => {
    const path = String(url);
    assert.equal(new Headers(options?.headers).get("Authorization"), "Bearer installation-token");
    const body = options?.body ? JSON.parse(String(options.body)) : undefined;
    calls.push({ url: path, body });
    if (path.includes("/contents/")) return new Response(manifest);
    if (path.includes("/pulls?") || path.includes("/matching-refs/")) return Response.json([]);
    if (path.endsWith("/pulls"))
      return Response.json({ html_url: "https://github.com/yhaggai/kitor/pull/999" });
    return Response.json({ sha: "a".repeat(40), tree: { sha: "b".repeat(40) } });
  };
  try {
    await openPR(
      { base_sha: "a".repeat(40), digest: "b".repeat(64), edits: { row: "corrected" } },
      "csv",
      "v1",
      "submission",
      { id: "public-id", name: "Contributor" },
      "installation-token"
    );
    const pr = calls.find((c) => c.url.endsWith("/pulls"))?.body;
    assert.ok(pr);
    assert.equal(pr.draft, true);
    assert.match(String(pr.body), /Contributor: \*\*Contributor\*\*/);
    assert.match(String(pr.body), /public-id/);
    const commit = calls.find((c) => c.url.endsWith("/git/commits"))?.body;
    assert.ok(commit);
    assert.equal(commit.author, undefined);
    assert.equal(commit.committer, undefined);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("GitHub redirects are rejected without forwarding bot credentials", async () => {
  const { github } = await import("../src/github");
  const originalFetch = globalThis.fetch;
  let requests = 0;
  globalThis.fetch = async (_url, options) => {
    requests++;
    // Workers supports manual/follow, unlike Node's additional error mode.
    assert.equal(options?.redirect, "manual");
    return new Response(null, { status: 302, headers: { Location: "https://example.com" } });
  };
  try {
    await assert.rejects(github("repos/yhaggai/kitor", "test-token"), /GitHub returned 302/);
    assert.equal(requests, 1);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

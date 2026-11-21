import test from "node:test";
import assert from "node:assert/strict";
import { issueCandidates, reportBuildIssues } from "./report-issues.mjs";

function snapshot(status = "changed") {
  return { rows: [{ gameId: "moomin", key: "steam:4463520", title: "Moomin", store: "steam", availability: "ok",
    latest: { buildId: "new", manifests: { "4463521": "new-manifest" } },
    supported: [{ platform: "windows", language: "he", patchVersion: "0.1.1", build_id: "old", depot_id: "4463521", manifest_id: "old-manifest", status: "different", fileCheck: { status, paths: ["Summer/catalog.bin"] } }],
  }] };
}

test("only confirmed changed patch inputs produce issues", () => {
  for (const status of ["unchanged", "unavailable", null]) assert.deepEqual(issueCandidates(snapshot(status)), []);
  const empty = snapshot(); empty.rows[0].supported[0].fileCheck.paths = [];
  assert.deepEqual(issueCandidates(empty), []);
  const failed = snapshot(); failed.rows[0].availability = "error";
  assert.deepEqual(issueCandidates(failed), []);
  const [issue] = issueCandidates(snapshot());
  assert.match(issue.body, /Summer\/catalog.bin/);
  assert.match(issue.body, /Supported build: old/);
  assert.match(issue.body, /Current build: new/);
});

test("repeated runs reuse open and closed issues without reopening them", async () => {
  for (const state of ["open", "closed"]) {
    const [candidate] = issueCandidates(snapshot());
    const result = await reportBuildIssues(snapshot(), "yhaggai/kitor", async method => {
      assert.equal(method, "GET");
      return [{ ...candidate, number: 1, state }];
    });
    assert.deepEqual(result, { created: 0, updated: 0 });
  }
});

test("new comparisons create distinct issues and changed reports update the open issue", async () => {
  const [candidate] = issueCandidates(snapshot());
  const calls = [];
  const result = await reportBuildIssues(snapshot(), "yhaggai/kitor", async (method, path, body) => {
    calls.push({ method, path, body });
    return method === "GET" ? [] : { number: 2, state: "open" };
  });
  assert.deepEqual(result, { created: 1, updated: 0 });
  assert.equal(calls[1].method, "POST");
  const newer = snapshot(); newer.rows[0].latest.manifests["4463521"] = "another-manifest";
  assert.notEqual(issueCandidates(newer)[0].marker, candidate.marker);
  const updates = [];
  await reportBuildIssues(snapshot(), "yhaggai/kitor", async (method, path, body) => {
    if (method === "GET") return [{ number: 2, state: "open", body: candidate.marker }];
    updates.push({ method, path, body }); return {};
  });
  assert.equal(updates[0].method, "PATCH");
  assert.equal(updates[0].path, "/repos/yhaggai/kitor/issues/2");
});

test("pagination finds an existing issue beyond the first page and ignores PR bodies", async () => {
  const [candidate] = issueCandidates(snapshot());
  let pages = 0;
  const result = await reportBuildIssues(snapshot(), "yhaggai/kitor", async (method, path) => {
    assert.equal(method, "GET"); pages++;
    return path.endsWith("page=1") ? Array.from({ length: 100 }, () => ({ pull_request: {}, body: candidate.body })) : [{ ...candidate, state: "closed" }];
  });
  assert.equal(pages, 2);
  assert.deepEqual(result, { created: 0, updated: 0 });
});

test("unchanged builds do not access GitHub and invalid destinations are rejected", async () => {
  assert.deepEqual(await reportBuildIssues(snapshot("unchanged"), "yhaggai/kitor", () => assert.fail()), { created: 0, updated: 0 });
  await assert.rejects(reportBuildIssues(snapshot(), "../elsewhere", () => assert.fail()), /Invalid GitHub repository/);
});


test("GOG comparisons report the matching baseline and current build", () => {
  const data = snapshot();
  const row = data.rows[0];
  row.store = "gog"; row.key = "gog:123:windows";
  row.supported[0].build_id = "gog-old";
  row.supported.unshift({ ...row.supported[0], build_id: "other-baseline", fileCheck: undefined });
  const [issue] = issueCandidates(data);
  assert.match(issue.title, /GOG windows build new/);
  assert.match(issue.body, /Supported build: gog-old/);
  assert.match(issue.body, /Builds: gog-old → new/);
  row.supported[1].status = "match";
  assert.deepEqual(issueCandidates(data), []);
});

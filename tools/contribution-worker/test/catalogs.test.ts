import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";
import { catalog, catalogIds, MAX_CATALOG_BYTES } from "../src/catalogs";
import { apply, validate } from "../src/csv";
import { findOpenPR, openPR } from "../src/github";
import type { Env } from "../src/shared";

const fixtures: {
  catalog: string;
  source: string;
  expected: string;
  edits: Record<string, string>;
  rejected: Record<string, string>;
}[] = JSON.parse(
  await readFile(new URL("../../../kitor/testdata/contribution-csv.json", import.meta.url), "utf8")
);

test("desktop and Worker agree on changes to every beta catalog", async () => {
  for (const { catalog: id, source, expected, edits, rejected } of fixtures) {
    const config = catalog(id);
    assert.equal(await apply(source, edits, config), expected, id);
    await assert.rejects(apply(source, rejected, config), { status: 400 }, id);
  }
});

test("all published catalog schemas parse, including the large Disco source and Moomin BOM", async () => {
  for (const id of catalogIds.filter((id) => id !== "drifter")) {
    const config = catalog(id);
    const bytes = await readFile(new URL("../../../" + config.path, import.meta.url));
    assert.ok(bytes.length <= MAX_CATALOG_BYTES, id);
    const source = bytes.toString("utf8");
    assert.equal(await apply(source, {}, config), source, id);
  }
});

test("only allowlisted catalog IDs are accepted", () => {
  assert.throws(
    () =>
      validate({
        catalog_id: "../patch.json",
        base_sha: "a".repeat(40),
        digest: "b".repeat(64),
        edits: { "2:demo": "תיקון" },
      }),
    { status: 400 }
  );
});

test("open draft reuse stays in the same catalog; beta PRs write only their CSV", async () => {
  const config = catalog("undertale");
  const repo = { full_name: "yhaggai/kitor" };
  const parent = "a".repeat(40);
  const id = "b".repeat(64);
  const open = {
    number: 71,
    state: "open",
    html_url: "https://github.com/yhaggai/kitor/pull/71",
    head: { ref: "kitor-contributions/" + id, sha: parent, repo },
    base: { ref: "v1", repo },
  };
  const env = {
    DB: {
      prepare: (sql: string) => {
        assert.match(sql, /catalog_id=\?/);
        return {
          bind: (_contributor: string, id: string) => ({
            all: async () => ({ results: id === "undertale" ? [{ url: open.html_url }] : [] }),
          }),
        };
      },
    },
  } as unknown as Env;
  const original = globalThis.fetch;
  let writes = 0;
  globalThis.fetch = async (url, options) => {
    const endpoint = String(url);
    const body = options?.body ? JSON.parse(String(options.body)) : undefined;
    if (endpoint.includes("/pulls?"))
      return Response.json(endpoint.includes("state=open") ? [open] : []);
    if (endpoint.includes("matching-refs")) return Response.json([]);
    if (endpoint.endsWith("/git/trees")) {
      assert.deepEqual(
        body.tree.map((entry: { path: string }) => entry.path),
        [config.path]
      );
      writes++;
    }
    if (endpoint.endsWith("/pulls")) {
      assert.equal(body.draft, true);
      assert.match(body.title, /Undertale/);
      return Response.json(open);
    }
    assert.ok(!endpoint.includes("/contents/"), "beta PRs don't fetch a binary payload manifest");
    return Response.json({ sha: parent, tree: { sha: parent } });
  };
  try {
    assert.equal(await findOpenPR(env, "user", "v1", "token", catalog("undertale-niqqud")), null);
    assert.equal((await findOpenPR(env, "user", "v1", "token", config))?.html_url, open.html_url);
    const result = await openPR(
      {
        catalog_id: config.id,
        base_sha: parent,
        digest: "c".repeat(64),
        edits: { "2:demo": "היי" },
      },
      "source csv",
      "v1",
      id,
      { id: "user", name: "Test Contributor" },
      "token"
    );
    assert.equal(result.url, open.html_url);
    assert.equal(writes, 1);
  } finally {
    globalThis.fetch = original;
  }
});

import test from "node:test";
import assert from "node:assert/strict";
import { handleManualCheck } from "./manual-check.mjs";
import { triggerCheck } from "./trigger-check.mjs";

const request = (token, action = "scan", method = "POST") => new Request("https://build.kitor.click/admin/check", {
  method, headers: token ? { Authorization: `Bearer ${token}` } : {},
  ...(method === "POST" ? { body: JSON.stringify({ action }) } : {}),
});

test("manual checks fail closed without a configured or matching token", async () => {
  for (const [configured, supplied] of [[undefined, undefined], ["secret", undefined], ["secret", "wrong!"], ["secret", "short"]]) {
    const response = await handleManualCheck(request(supplied), configured, () => assert.fail("must not run"));
    assert.equal(response.status, 401);
  }
});

test("manual checks reject GET, malformed JSON, and unknown actions", async () => {
  assert.equal((await handleManualCheck(request("secret", "scan", "GET"), "secret", () => assert.fail())).status, 405);
  assert.equal((await handleManualCheck(request("secret", "delete"), "secret", () => assert.fail())).status, 400);
  const malformed = new Request("https://build.kitor.click/admin/check", { method: "POST", headers: { Authorization: "Bearer secret" }, body: "{" });
  assert.equal((await handleManualCheck(malformed, "secret", () => assert.fail())).status, 400);
});

test("authenticated checks await each allowed operation and report success", async () => {
  for (const action of ["scan", "gog", "alerts"]) {
    let completed = false;
    const response = await handleManualCheck(request("secret", action), "secret", async received => {
      assert.equal(received, action);
      await Promise.resolve();
      completed = true;
    });
    assert.equal(completed, true);
    assert.deepEqual(await response.json(), { ok: true, action });
  }
});

test("trigger authenticates to the fixed monitor URL and refuses redirects", async () => {
  await triggerCheck("alerts", "secret", async (url, options) => {
    assert.equal(url, "https://build.kitor.click/admin/check");
    assert.equal(options.redirect, "error");
    assert.equal(options.headers.Authorization, "Bearer secret");
    return handleManualCheck(new Request(url, options), "secret", async action => assert.equal(action, "alerts"));
  });
});

test("trigger surfaces failed operations without leaking response bodies or credentials", async () => {
  await assert.rejects(triggerCheck("scan", "secret", async () => new Response("private secret details", { status: 500 })), /^Error: Monitor scan returned HTTP 500$/);
  await assert.rejects(triggerCheck("scan", undefined, () => assert.fail()), /not configured/);
  await assert.rejects(triggerCheck("unknown", "secret", () => assert.fail()), /Invalid monitor action/);
});

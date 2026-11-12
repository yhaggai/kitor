import assert from "node:assert/strict";
import { createHmac } from "node:crypto";
import { readFile } from "node:fs/promises";
import { test } from "node:test";
import { convertV4MiniflareOptions, Miniflare } from "miniflare";
import { records } from "../src/csv";
import { drainNotifications, releaseContains, webhook } from "../src/notifications";
import { saveConsent, unsubscribe } from "../src/preferences";
import { type Env, hash } from "../src/shared";

const id = "a".repeat(64);
const secret = "test-webhook-secret";
const event = {
  action: "closed",
  repository: { full_name: "yhaggai/kitor" },
  pull_request: {
    number: 25,
    merged: true,
    merge_commit_sha: "b".repeat(40),
    head: { ref: `kitor-contributions/${id}` },
    base: { ref: "v1", repo: { full_name: "yhaggai/kitor" } },
  },
};
function request(value: unknown, kind = "pull_request", signed = true) {
  const body = JSON.stringify(value);
  return new Request("http://localhost/webhooks/github", {
    method: "POST",
    body,
    headers: {
      "x-github-event": kind,
      "x-hub-signature-256":
        "sha256=" +
        (signed ? createHmac("sha256", secret).update(body).digest("hex") : "0".repeat(64)),
    },
  });
}
test("signed webhook, merge matching, private email, deduplication and retry queue with real D1", async () => {
  const mf = new Miniflare(
    convertV4MiniflareOptions({
      workers: [
        {
          modules: true,
          script: "export default {fetch(){return new Response('ok')}}",
          compatibilityDate: "2026-11-01",
          d1Databases: ["DB"],
        },
      ],
    })
  );
  try {
    const db = await mf.getD1Database("DB");
    for (const file of ["0001_identity.sql", "0002_notifications.sql", "0003_email_consent.sql"])
      for (const sql of (await readFile(`migrations/${file}`, "utf8"))
        .split(";")
        .filter((s) => s.trim()))
        await db.prepare(sql).run();
    const env = {
      DB: db,
      PUBLIC_URL: "http://localhost",
      GITHUB_WEBHOOK_SECRET: secret,
      NOTIFICATIONS_ENABLED: "true",
      RESEND_API_KEY: "test",
      NOTIFICATION_FROM: "Kitor <notifications@kitor.click>",
    } as unknown as Env;
    await db
      .prepare(
        "INSERT INTO contributors(id,google_sub,name,created_at,email) VALUES ('u','sub','Name',0,'private@example.com')"
      )
      .run();
    await db
      .prepare(
        "INSERT INTO submissions(id,contributor_id,status,lease_until,url,created_at) VALUES (?,'u','complete',0,'https://github.com/yhaggai/kitor/pull/25',0)"
      )
      .bind(id)
      .run();
    await assert.rejects(webhook(request(event, "pull_request", false), env), /signature/);
    await webhook(
      request({ ...event, pull_request: { ...event.pull_request, merged: false } }),
      env
    );
    await webhook(request({ ...event, repository: { full_name: "other/repo" } }), env);
    await webhook(request({ ...event, pull_request: { ...event.pull_request, number: 26 } }), env);
    assert.equal(
      (await db.prepare("SELECT COUNT(*) AS n FROM notifications").first<{ n: number }>())!.n,
      0
    );
    await Promise.all([webhook(request(event), env), webhook(request(event), env)]);
    assert.equal(
      (await db.prepare("SELECT COUNT(*) AS n FROM notifications").first<{ n: number }>())!.n,
      1
    );
    await drainNotifications(env, async () => {
      throw new Error("Must not send without opt-in");
    });
    assert.equal(
      (await db.prepare("SELECT attempts FROM notifications").first<{ attempts: number }>())!
        .attempts,
      0
    );
    await saveConsent(env, id, "u", true);
    let sends = 0;
    const send = async (_env: unknown, msg: { to: string; id: string }) => {
      sends++;
      assert.equal(msg.to, "private@example.com");
      assert.equal(msg.id, `approved/${id}`);
      return "delivery";
    };
    await Promise.all([drainNotifications(env, send), drainNotifications(env, send)]);
    await webhook(request(event), env);
    await drainNotifications(env, send);
    assert.equal(sends, 1);
    assert.equal(
      (await db.prepare("SELECT status FROM notifications").first<{ status: string }>())!.status,
      "sent"
    );
    await db
      .prepare(
        "UPDATE notifications SET status='pending',first_attempt=NULL,recipient=NULL,sender=NULL,sent_at=NULL"
      )
      .run();
    await drainNotifications(env, async () => {
      throw new Error("timeout");
    });
    assert.equal(
      (await db.prepare("SELECT status FROM notifications").first<{ status: string }>())!.status,
      "pending"
    );
    await db
      .prepare("UPDATE notifications SET first_attempt=?,next_attempt=0")
      .bind(Math.floor(Date.now() / 1000) - 24 * 3600)
      .run();
    await drainNotifications(env, send);
    assert.equal(
      (await db.prepare("SELECT status FROM notifications").first<{ status: string }>())!.status,
      "needs_review"
    );
    assert.equal(sends, 1);
    await db
      .prepare("UPDATE notifications SET status='pending',first_attempt=NULL,next_attempt=0")
      .run();
    await db
      .prepare("INSERT OR REPLACE INTO notification_budget(day,count) VALUES (?,90)")
      .bind(new Date().toISOString().slice(0, 10))
      .run();
    await drainNotifications(env, send);
    assert.equal(sends, 1, "daily budget prevents sending");
    await db.prepare("DELETE FROM notification_budget").run();
    await db.prepare("UPDATE contributors SET email=NULL").run();
    await drainNotifications(env, send);
    assert.equal(sends, 1, "old profiles wait for a verified email");
    await webhook(
      request(
        {
          action: "published",
          repository: event.repository,
          release: { draft: false, prerelease: false, tag_name: "patch-drifter-he-v1" },
        },
        "release"
      ),
      env
    );
    assert.equal(
      (await db.prepare("SELECT COUNT(*) AS n FROM patch_releases").first<{ n: number }>())!.n,
      1
    );
    const token = (await db
      .prepare("SELECT unsubscribe_token FROM contributors")
      .first<{ unsubscribe_token: string }>())!.unsubscribe_token;
    await unsubscribe(
      new Request(`http://localhost/notifications/unsubscribe?token=${token}`),
      env
    );
    assert.equal(
      (await db.prepare("SELECT email_opt_in FROM submissions").first<{ email_opt_in: number }>())!
        .email_opt_in,
      1,
      "GET does not unsubscribe (mail scanners)"
    );
    await unsubscribe(
      new Request(`http://localhost/notifications/unsubscribe?token=${token}`, { method: "POST" }),
      env
    );
    assert.equal(
      (await db.prepare("SELECT email_opt_in FROM submissions").first<{ email_opt_in: number }>())!
        .email_opt_in,
      0
    );
    assert.equal(
      (await db.prepare("SELECT status FROM notifications").first<{ status: string }>())!.status,
      "cancelled"
    );
    await drainNotifications(env, send);
    assert.equal(sends, 1);
    await assert.rejects(
      unsubscribe(new Request("http://localhost/notifications/unsubscribe?token=bad"), env)
    );
  } finally {
    await mf.dispose();
  }
});
test("release notification requires ancestry, published assets and unchanged correction in tagged CSV", async () => {
  const csv = await readFile("../../patches-workspace/drifter/he/payload/translation.csv", "utf8");
  const row = records(csv)[3];
  const key = await hash(JSON.stringify(row.fields.slice(0, 5)));
  const original = globalThis.fetch;
  let ancestry = "ahead",
    draft = false;
  globalThis.fetch = (async (url) => {
    const path = String(url);
    if (path.includes("/releases/"))
      return Response.json({
        draft,
        prerelease: false,
        published_at: "today",
        assets: [{ name: "patch.zip", size: 100 }],
      });
    if (path.includes("/compare/")) return Response.json({ status: ancestry });
    if (path.includes("patch.json"))
      return new Response(
        JSON.stringify({
          version: "1",
          filename: "patch.zip",
          files: [{ path: "translation.csv", source: { sha256: await hash(csv) } }],
        })
      );
    return new Response(csv);
  }) as typeof fetch;
  try {
    assert.equal(
      await releaseContains(
        "patch-drifter-he-v1",
        "b".repeat(40),
        { [key]: row.fields[5] },
        "test"
      ),
      true
    );
    assert.equal(
      await releaseContains("patch-drifter-he-v1", "b".repeat(40), { [key]: "different" }, "test"),
      false
    );
    ancestry = "behind";
    assert.equal(
      await releaseContains(
        "patch-drifter-he-v1",
        "b".repeat(40),
        { [key]: row.fields[5] },
        "test"
      ),
      false
    );
    ancestry = "ahead";
    draft = true;
    assert.equal(
      await releaseContains(
        "patch-drifter-he-v1",
        "b".repeat(40),
        { [key]: row.fields[5] },
        "test"
      ),
      false
    );
  } finally {
    globalThis.fetch = original;
  }
});

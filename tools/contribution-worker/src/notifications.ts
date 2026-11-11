import { apply } from "./csv";
import { sendEmail } from "./email";
import { github, installation, source } from "./github";
import { type Env, HttpError, hash, json, now } from "./shared";

const REPO = "yhaggai/kitor";
const ROOT = "patches-workspace/drifter/he/";

export async function webhook(req: Request, env: Env): Promise<Response> {
  if (!env.GITHUB_WEBHOOK_SECRET) throw new HttpError(503, "Webhook is not configured");
  const signature = req.headers.get("x-hub-signature-256");
  if (!signature || !/^sha256=[a-f0-9]{64}$/.test(signature))
    throw new HttpError(401, "Invalid webhook signature");
  const reader = req.body?.getReader();
  if (!reader) throw new HttpError(400, "Missing webhook body");
  const chunks: Uint8Array[] = [];
  let length = 0;
  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    length += value.length;
    if (length > 1_000_000) {
      await reader.cancel();
      throw new HttpError(413, "Webhook too large");
    }
    chunks.push(value);
  }
  const bytes = new Uint8Array(length);
  let offset = 0;
  for (const chunk of chunks) {
    bytes.set(chunk, offset);
    offset += chunk.length;
  }
  const key = await crypto.subtle.importKey(
    "raw",
    new TextEncoder().encode(env.GITHUB_WEBHOOK_SECRET),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["verify"]
  );
  const sig = Uint8Array.from(signature.slice(7).match(/../g)!, (x) => parseInt(x, 16));
  if (!(await crypto.subtle.verify("HMAC", key, sig, bytes)))
    throw new HttpError(401, "Invalid webhook signature");
  let event: any;
  try {
    event = JSON.parse(new TextDecoder().decode(bytes));
  } catch {
    throw new HttpError(400, "Invalid webhook JSON");
  }
  if (!event || typeof event !== "object") throw new HttpError(400, "Invalid webhook");
  if (req.headers.get("x-github-event") === "ping") return json({ ok: true });
  if (event.repository?.full_name !== REPO) return json({ ignored: true });
  if (
    req.headers.get("x-github-event") === "pull_request" &&
    event.action === "closed" &&
    event.pull_request?.merged === true
  ) {
    const pr = event.pull_request;
    const id = pr.head?.ref?.match(/^kitor-contributions\/([a-f0-9]{64})$/)?.[1];
    if (
      !id ||
      pr.base?.repo?.full_name !== REPO ||
      pr.base?.ref !== "v1" ||
      !/^[a-f0-9]{40}$/.test(pr.merge_commit_sha || "") ||
      !Number.isSafeInteger(pr.number)
    )
      return json({ ignored: true });
    const url = `https://github.com/${REPO}/pull/${pr.number}`;
    const submission = await env.DB.prepare("SELECT status,url FROM submissions WHERE id=?")
      .bind(id)
      .first<{ status: string; url: string }>();
    if (!submission) return json({ ignored: true });
    if (submission.status !== "complete")
      throw new HttpError(503, "Submission is still processing; redeliver this event");
    if (submission.url !== url) return json({ ignored: true });
    const inProgress = await env.DB.prepare(
      "SELECT id FROM submissions WHERE url=? AND status='pending' AND lease_until>? LIMIT 1"
    )
      .bind(url, now())
      .first();
    if (inProgress)
      throw new HttpError(503, "Submission is still processing; redeliver this event");
    const message = `Thank you for improving The Drifter's Hebrew translation! Your contribution has been reviewed and merged.\n\n${url}\n\nWe'll email you again when a published patch includes your corrections.\n\nThe Kitor team`;
    await env.DB.batch([
      env.DB.prepare(
        "UPDATE submissions SET merge_sha=COALESCE(merge_sha,?) WHERE url=? AND status='complete'"
      ).bind(pr.merge_commit_sha, url),
      env.DB.prepare(`INSERT OR IGNORE INTO notifications(id,submission_id,kind,subject,message)
        SELECT 'approved/' || id,id,'approved',?,? FROM submissions WHERE url=? AND status='complete'`).bind(
        "Your Kitor contribution was approved 🎉",
        message,
        url
      ),
    ]);
  }
  if (
    req.headers.get("x-github-event") === "release" &&
    event.action === "published" &&
    event.release?.draft === false &&
    event.release?.prerelease === false &&
    /^patch-drifter-he-v[A-Za-z0-9][A-Za-z0-9._-]{0,100}$/.test(event.release?.tag_name || "")
  ) {
    await env.DB.prepare("INSERT OR IGNORE INTO patch_releases(tag,created_at) VALUES (?,?)")
      .bind(event.release.tag_name, now())
      .run();
  }
  return json({ ok: true });
}
function queue(
  env: Env,
  id: string,
  submission: string,
  kind: string,
  subject: string,
  text: string
) {
  return env.DB.prepare(
    "INSERT OR IGNORE INTO notifications(id,submission_id,kind,subject,message) VALUES (?,?,?,?,?)"
  ).bind(id, submission, kind, subject, text);
}
export async function releaseContains(
  tag: string,
  merge: string,
  edits: Record<string, string>,
  token: string
): Promise<boolean> {
  const release = await github(`repos/${REPO}/releases/tags/${encodeURIComponent(tag)}`, token);
  if (release.draft || release.prerelease || !release.published_at || !release.assets?.length)
    return false;
  const compare = await github(
    `repos/${REPO}/compare/${merge}...${encodeURIComponent(tag)}`,
    token
  );
  if (!["ahead", "identical"].includes(compare.status)) return false;
  const manifest = JSON.parse(await source(ROOT + "patch.json", tag, token));
  if (
    `patch-drifter-he-v${manifest.version}` !== tag ||
    !release.assets.some(
      (asset: { name: string; size: number }) => asset.name === manifest.filename && asset.size > 0
    )
  )
    return false;
  const csv = await source(ROOT + "payload/translation.csv", tag, token);
  const entry = manifest.files?.find((file: { path: string }) => file.path === "translation.csv");
  if (!entry?.source?.sha256 || entry.source.sha256 !== (await hash(csv))) return false;
  try {
    return (await apply(csv, edits)) === csv;
  } catch {
    return false;
  }
}
export async function reconcileReleases(env: Env) {
  if (env.NOTIFICATIONS_ENABLED !== "true" || !env.GITHUB_PRIVATE_KEY) return;
  const rows =
    await env.DB.prepare(`SELECT s.id,s.merge_sha,s.edits_json,r.tag FROM submissions s CROSS JOIN patch_releases r
    LEFT JOIN release_checks c ON c.submission_id=s.id AND c.tag=r.tag
    WHERE s.merge_sha IS NOT NULL AND s.edits_json IS NOT NULL AND s.released_tag IS NULL
    AND (c.checked_at IS NULL OR c.checked_at<?)
    ORDER BY COALESCE(c.checked_at,0),r.created_at DESC LIMIT 1`)
      .bind(now() - 86400)
      .all<{ id: string; merge_sha: string; edits_json: string; tag: string }>();
  if (!rows.results.length) return;
  try {
    const token = await installation(env);
    for (const row of rows.results) {
      await env.DB.prepare("UPDATE submissions SET release_checked_at=? WHERE id=?")
        .bind(now(), row.id)
        .run();
      const tag = row.tag;
      {
        const included = await releaseContains(
          tag,
          row.merge_sha,
          JSON.parse(row.edits_json),
          token
        );
        await env.DB.prepare(
          "INSERT INTO release_checks(submission_id,tag,checked_at) VALUES (?,?,?) ON CONFLICT(submission_id,tag) DO UPDATE SET checked_at=excluded.checked_at"
        )
          .bind(row.id, tag, now())
          .run();
        if (!included) continue;
        await env.DB.batch([
          queue(
            env,
            `released/${row.id}`,
            row.id,
            "released",
            "Your Kitor corrections are available in a new patch 🎉",
            `Your corrections to The Drifter's Hebrew translation are included in patch ${tag.slice("patch-drifter-he-v".length)}.\n\nhttps://github.com/${REPO}/releases/tag/${tag}\n\nThank you for helping the community!\nThe Kitor team`
          ),
          env.DB.prepare("UPDATE submissions SET released_tag=? WHERE id=?").bind(tag, row.id),
        ]);
        break;
      }
    }
  } catch {
    console.error("Patch notification reconciliation failed; will retry");
  }
}

export async function drainNotifications(env: Env, send = sendEmail) {
  if (env.NOTIFICATIONS_ENABLED !== "true" || !env.RESEND_API_KEY || !env.NOTIFICATION_FROM) return;
  await env.DB.prepare(
    "UPDATE notifications SET status='needs_review' WHERE status='pending' AND first_attempt IS NOT NULL AND first_attempt<? AND lease_until<?"
  )
    .bind(now() - 23 * 3600, now())
    .run();
  for (let i = 0; i < 4; i++) {
    const row =
      await env.DB.prepare(`UPDATE notifications SET lease_until=?, recipient=COALESCE(recipient,(SELECT c.email FROM submissions s JOIN contributors c ON c.id=s.contributor_id WHERE s.id=submission_id)), sender=COALESCE(sender,?)
      WHERE id=(SELECT n.id FROM notifications n JOIN submissions s ON s.id=n.submission_id JOIN contributors c ON c.id=s.contributor_id
      WHERE n.status='pending' AND n.next_attempt<=? AND n.lease_until<? AND c.email IS NOT NULL AND s.email_opt_in=1 AND c.email_unsubscribed=0
      AND (n.kind='approved' OR EXISTS(SELECT 1 FROM notifications a WHERE a.submission_id=n.submission_id AND a.kind='approved' AND a.status='sent'))
      ORDER BY n.id LIMIT 1) RETURNING *`)
        .bind(now() + 300, env.NOTIFICATION_FROM, now(), now())
        .first<{
          id: string;
          recipient: string;
          sender: string;
          subject: string;
          message: string;
          attempts: number;
        }>();
    if (!row) break;
    const day = new Date().toISOString().slice(0, 10);
    const budget = await env.DB.prepare(
      "INSERT INTO notification_budget(day,count) VALUES (?,1) ON CONFLICT(day) DO UPDATE SET count=count+1 WHERE count<90 RETURNING count"
    )
      .bind(day)
      .first();
    if (!budget) {
      await env.DB.prepare("UPDATE notifications SET lease_until=0 WHERE id=?").bind(row.id).run();
      break;
    }
    await env.DB.prepare(
      "UPDATE notifications SET first_attempt=COALESCE(first_attempt,?),attempts=attempts+1 WHERE id=?"
    )
      .bind(now(), row.id)
      .run();
    const consent = await env.DB.prepare(
      "SELECT c.unsubscribe_token FROM notifications n JOIN submissions s ON s.id=n.submission_id JOIN contributors c ON c.id=s.contributor_id WHERE n.id=? AND s.email_opt_in=1 AND c.email_unsubscribed=0 AND n.status='pending'"
    )
      .bind(row.id)
      .first<{ unsubscribe_token: string }>();
    if (!consent?.unsubscribe_token) {
      await env.DB.prepare("UPDATE notifications SET status='cancelled',lease_until=0 WHERE id=?")
        .bind(row.id)
        .run();
      continue;
    }
    // Freeze the whole body so provider idempotency retries have identical payloads.
    if (!row.message.includes("Stop contribution emails:")) {
      row.message += `\n\nStop contribution emails: ${env.PUBLIC_URL}/notifications/unsubscribe?token=${consent.unsubscribe_token}`;
      await env.DB.prepare("UPDATE notifications SET message=? WHERE id=?")
        .bind(row.message, row.id)
        .run();
    }
    try {
      const id = await send(
        { ...env, NOTIFICATION_FROM: row.sender },
        { id: row.id, to: row.recipient, subject: row.subject, text: row.message }
      );
      await env.DB.prepare(
        "UPDATE notifications SET status='sent',provider_id=?,sent_at=?,lease_until=0 WHERE id=?"
      )
        .bind(id, now(), row.id)
        .run();
    } catch {
      await env.DB.prepare("UPDATE notifications SET lease_until=0,next_attempt=? WHERE id=?")
        .bind(now() + 3600, row.id)
        .run();
      console.error("Notification delivery deferred");
    }
  }
}

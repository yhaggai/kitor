import { type Env, HttpError, now, random } from "./shared";

export async function saveConsent(
  env: Env,
  submission: string,
  contributor: string,
  optedIn: boolean
) {
  await env.DB.batch([
    env.DB.prepare(
      "UPDATE submissions SET email_opt_in=?,email_consent_at=? WHERE id=? AND contributor_id=?"
    ).bind(optedIn ? 1 : 0, optedIn ? now() : null, submission, contributor),
    env.DB.prepare(
      "UPDATE contributors SET unsubscribe_token=COALESCE(unsubscribe_token,?),email_unsubscribed=CASE WHEN ?=1 THEN 0 ELSE email_unsubscribed END WHERE id=?"
    ).bind(random(), optedIn ? 1 : 0, contributor),
  ]);
}
export async function unsubscribe(req: Request, env: Env): Promise<Response> {
  if (!["GET", "POST"].includes(req.method)) throw new HttpError(405, "Method not allowed");
  const token = new URL(req.url).searchParams.get("token") || "";
  if (!/^[a-f0-9]{64}$/.test(token)) throw new HttpError(400, "Invalid unsubscribe link");
  const contributor = await env.DB.prepare("SELECT id FROM contributors WHERE unsubscribe_token=?")
    .bind(token)
    .first<{ id: string }>();
  if (!contributor) throw new HttpError(400, "Invalid unsubscribe link");
  if (req.method === "POST") {
    await env.DB.batch([
      env.DB.prepare("UPDATE contributors SET email_unsubscribed=1 WHERE id=?").bind(
        contributor.id
      ),
      env.DB.prepare("UPDATE submissions SET email_opt_in=0 WHERE contributor_id=?").bind(
        contributor.id
      ),
      env.DB.prepare(
        "UPDATE notifications SET status='cancelled' WHERE status IN ('pending','needs_review') AND submission_id IN (SELECT id FROM submissions WHERE contributor_id=?)"
      ).bind(contributor.id),
    ]);
  }
  const content =
    req.method === "POST"
      ? "<h1>Contribution emails stopped</h1><p>You won’t receive further contribution emails. An email already being delivered may still arrive. Your contributions are unchanged.</p>"
      : `<h1>Stop contribution emails?</h1><p>This stops updates for all your contributions. You can still contribute without emails.</p><form method="post" action="/notifications/unsubscribe?token=${token}"><button type="submit">Stop contribution emails</button></form>`;
  return new Response(
    `<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>Kitor email preferences</title><body>${content}</body></html>`,
    {
      headers: {
        "Content-Type": "text/html; charset=utf-8",
        "Cache-Control": "no-store",
        "Referrer-Policy": "no-referrer",
        "X-Robots-Tag": "noindex",
        "Content-Security-Policy": "default-src 'none'; form-action 'self'; frame-ancestors 'none'",
      },
    }
  );
}

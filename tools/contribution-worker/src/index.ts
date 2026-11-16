import { auth, user } from "./auth";
import { catalog, catalogIds } from "./catalogs";
import { apply, validate } from "./csv";
import { appendPR, canonical, findOpenPR, installation, openPR } from "./github";
import { drainNotifications, reconcileReleases, webhook } from "./notifications";
import { saveConsent, setContributionEmails, unsubscribe } from "./preferences";
import { body, type Env, HttpError, hash, json, limit, now, random } from "./shared";
export default {
  async fetch(req: Request, env: Env): Promise<Response> {
    let stage = "request";
    try {
      const path = new URL(req.url).pathname;
      if (path === "/notifications/unsubscribe") return await unsubscribe(req, env);
      if (req.method === "POST" && req.headers.has("origin"))
        throw new HttpError(403, "Use the Kitor desktop app");
      if (path === "/health")
        return json({
          service: "kitor-contribution-worker",
          configured: !!env.GITHUB_PRIVATE_KEY,
          catalogs: catalogIds,
          google_configured: !!env.GOOGLE_CLIENT_ID && !!env.GOOGLE_CLIENT_SECRET,
        });
      if (path === "/webhooks/github" && req.method === "POST") return await webhook(req, env);
      const login = await auth(req, env, path);
      if (login) return login;
      if (path === "/notifications/preferences") {
        const contributor = await user(req, env);
        if (req.method === "POST") {
          const { enabled } = await body(req, 1024);
          if (typeof enabled !== "boolean") throw new HttpError(400, "Invalid email preference");
          await setContributionEmails(env, contributor.id, enabled);
          return json({ enabled });
        }
        if (req.method !== "GET") throw new HttpError(405, "Method not allowed");
        const preference = await env.DB.prepare(
          "SELECT email_unsubscribed FROM contributors WHERE id=?"
        )
          .bind(contributor.id)
          .first<{ email_unsubscribed: number }>();
        return json({ enabled: preference?.email_unsubscribed === 0 });
      }
      if (path === "/submissions" && req.method === "POST") {
        stage = "session";
        const contributor = await user(req, env);
        const input = await body(req);
        validate(input);
        const config = catalog(input.catalog_id);
        const scope = config.id === "drifter" ? "" : config.id + ":";
        const fingerprint = await hash(
          `${scope}${contributor.id}:${input.base_sha}:${input.digest}`
        );
        const owner = random();
        const lock = await env.DB.prepare(
          "INSERT INTO contribution_leases(contributor_id,owner,expires_at) VALUES (?,?,?) ON CONFLICT(contributor_id) DO UPDATE SET owner=excluded.owner,expires_at=excluded.expires_at WHERE contribution_leases.expires_at<? RETURNING owner"
        )
          .bind(contributor.id, owner, now() + 600, now())
          .first();
        if (!lock)
          throw new HttpError(409, "Your other submission is still processing. Retry shortly.");
        let pendingId: string | undefined;
        try {
          await limit(env, `submit:${contributor.id}`, 20, 3600);
          stage = "bot-authentication";
          const token = await installation(env);
          stage = "canonical-source";
          let { branch, csv: original } = await canonical(input.base_sha, token, config);
          let csv = await apply(original, input.edits, config);
          if (csv === original) throw new HttpError(400, "No corrections to submit");
          original = "";
          if ((await hash(csv)) !== input.digest)
            throw new HttpError(409, "Preview differs from canonical corrections");
          stage = "github-pr";
          const open = await findOpenPR(env, contributor.id, branch, token, config);
          if (open) csv = "";
          let id = fingerprint;
          for (;;) {
            const previous = await env.DB.prepare(
              "SELECT url FROM submissions WHERE id=? AND status='complete'"
            )
              .bind(id)
              .first<{ url: string }>();
            if (!previous) break;
            if (open?.html_url === previous.url) return json({ url: previous.url, existing: true });
            id = await hash(`${fingerprint}:${id}`);
          }
          const lease = await env.DB.prepare(
            "INSERT INTO submissions(id,contributor_id,status,lease_until,created_at,url,catalog_id) VALUES (?,?,'pending',?,?,?,?) ON CONFLICT(id) DO UPDATE SET status='pending',lease_until=excluded.lease_until,url=excluded.url WHERE submissions.status!='complete' AND submissions.lease_until<? RETURNING id"
          )
            .bind(id, contributor.id, now() + 600, now(), open?.html_url ?? null, config.id, now())
            .first();
          if (!lease)
            throw new HttpError(409, "This submission is already processing. Retry shortly");
          pendingId = id;
          if (config.id === "drifter" || config.id === "drifter-logical")
            await saveConsent(env, id, contributor.id, input.email_opt_in === true);
          const result = open
            ? await appendPR(open, input, token)
            : await openPR(input, csv, branch, id, contributor, token);
          await env.DB.prepare(
            "UPDATE submissions SET status='complete',url=?,edits_json=?,lease_until=0 WHERE id=?"
          )
            .bind(result.url, JSON.stringify(input.edits), id)
            .run();
          return json(result);
        } catch (e) {
          if (pendingId)
            await env.DB.prepare("UPDATE submissions SET status='failed',lease_until=0 WHERE id=?")
              .bind(pendingId)
              .run();
          throw e;
        } finally {
          await env.DB.prepare("DELETE FROM contribution_leases WHERE contributor_id=? AND owner=?")
            .bind(contributor.id, owner)
            .run();
        }
      }
      throw new HttpError(404, "Not found");
    } catch (e) {
      console.error("Contribution failure", {
        stage,
        type: e instanceof Error ? e.name : "unknown",
      });
      return json(
        {
          error:
            e instanceof HttpError ? e.message : "Request could not be completed. Retry shortly",
        },
        e instanceof HttpError ? e.status : 500
      );
    }
  },
  async scheduled(_event: ScheduledController, env: Env) {
    await reconcileReleases(env);
    await drainNotifications(env);
    await env.DB.batch([
      env.DB.prepare("DELETE FROM sessions WHERE expires_at<?").bind(now()),
      env.DB.prepare("DELETE FROM login_attempts WHERE expires_at<?").bind(now()),
      env.DB.prepare("DELETE FROM rate_limits WHERE expires_at<?").bind(now()),
    ]);
  },
};

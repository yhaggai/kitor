import { auth, user } from "./auth";
import { apply, validate } from "./csv";
import { canonical, installation, openPR } from "./github";
import { drainNotifications, reconcileReleases, webhook } from "./notifications";
import { body, type Env, HttpError, hash, json, limit, now } from "./shared";
export default {
  async fetch(req: Request, env: Env): Promise<Response> {
    let stage = "request";
    try {
      const path = new URL(req.url).pathname;
      if (req.method === "POST" && req.headers.has("origin"))
        throw new HttpError(403, "Use the Kitor desktop app");
      if (path === "/health")
        return json({
          service: "kitor-contribution-worker",
          configured: !!env.GITHUB_PRIVATE_KEY,
          google_configured: !!env.GOOGLE_CLIENT_ID && !!env.GOOGLE_CLIENT_SECRET,
        });
      if (path === "/webhooks/github" && req.method === "POST") return webhook(req, env);
      const login = await auth(req, env, path);
      if (login) return login;
      if (path === "/submissions" && req.method === "POST") {
        stage = "session";
        const contributor = await user(req, env);
        const input = await body(req);
        validate(input);
        const id = await hash(`${contributor.id}:${input.base_sha}:${input.digest}`);
        const previous = await env.DB.prepare(
          "SELECT url FROM submissions WHERE id=? AND status='complete'"
        )
          .bind(id)
          .first<{ url: string }>();
        if (previous) return json({ url: previous.url, existing: true });
        await limit(env, `submit:${contributor.id}`, 20, 3600);
        stage = "submission-lease";
        const lease = await env.DB.prepare(
          "INSERT INTO submissions(id,contributor_id,status,lease_until,created_at) VALUES (?,?,'pending',?,?) ON CONFLICT(id) DO UPDATE SET status='pending',lease_until=excluded.lease_until WHERE submissions.status!='complete' AND submissions.lease_until<? RETURNING id"
        )
          .bind(id, contributor.id, now() + 600, now(), now())
          .first();
        if (!lease)
          throw new HttpError(409, "This submission is already processing. Retry shortly");
        try {
          stage = "bot-authentication";
          const token = await installation(env);
          stage = "canonical-source";
          const base = await canonical(input.base_sha, token);
          const csv = await apply(base.csv, input.edits);
          if (csv === base.csv) throw new HttpError(400, "No corrections to submit");
          if ((await hash(csv)) !== input.digest)
            throw new HttpError(409, "Preview differs from canonical corrections");
          stage = "github-pr";
          const result = await openPR(input, csv, base.branch, id, contributor, token);
          await env.DB.prepare(
            "UPDATE submissions SET status='complete',url=?,edits_json=?,lease_until=0 WHERE id=?"
          )
            .bind(result.url, JSON.stringify(input.edits), id)
            .run();
          return json(result);
        } catch (e) {
          await env.DB.prepare("UPDATE submissions SET status='failed',lease_until=0 WHERE id=?")
            .bind(id)
            .run();
          throw e;
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

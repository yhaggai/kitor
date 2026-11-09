import { createRemoteJWKSet, type JWTVerifyGetKey, jwtVerify } from "jose";
import {
  body,
  type Contributor,
  challenge,
  type Env,
  HttpError,
  hash,
  json,
  limit,
  now,
  random,
} from "./shared";

const keys = createRemoteJWKSet(new URL("https://www.googleapis.com/oauth2/v3/certs"));
export async function identity(
  token: string,
  env: Env,
  nonce: string,
  keySet: JWTVerifyGetKey = keys
): Promise<{ sub: string; name: string }> {
  const { payload } = await jwtVerify(token, keySet, {
    issuer: ["https://accounts.google.com", "accounts.google.com"],
    audience: env.GOOGLE_CLIENT_ID,
    algorithms: ["RS256"],
    maxTokenAge: "10m",
  });
  if (
    payload.nonce !== nonce ||
    !payload.sub ||
    payload.email_verified !== true ||
    (payload.azp && payload.azp !== env.GOOGLE_CLIENT_ID)
  )
    throw new HttpError(401, "Google identity could not be verified");
  const name = typeof payload.name === "string" ? payload.name.trim() : "";
  if (
    !name ||
    name.length > 100 ||
    [...name].some((c) => c.charCodeAt(0) < 32 || c.charCodeAt(0) === 127)
  )
    throw new HttpError(400, "Google profile needs a valid display name");
  return { sub: payload.sub, name };
}
export async function user(req: Request, env: Env): Promise<Contributor> {
  const token = req.headers.get("authorization")?.match(/^Bearer ([a-f0-9]{64})$/)?.[1];
  if (!token) throw new HttpError(401, "Sign in with Google to contribute");
  const row = await env.DB.prepare(
    "SELECT c.id,c.name FROM sessions s JOIN contributors c ON c.id=s.contributor_id WHERE s.token_hash=? AND s.expires_at>?"
  )
    .bind(await hash(token), now())
    .first<Contributor>();
  if (!row) throw new HttpError(401, "Your session expired. Sign in again");
  return row;
}
const message = (text: string) =>
  new Response(
    `<!doctype html><meta charset="utf-8"><title>Kitor sign-in</title><h1>${text}</h1><p>You can close this tab and return to Kitor.</p>`,
    {
      headers: {
        "Content-Type": "text/html; charset=utf-8",
        "Cache-Control": "no-store",
        "Content-Security-Policy": "default-src 'none'; frame-ancestors 'none'",
        "Referrer-Policy": "no-referrer",
      },
    }
  );
export async function auth(req: Request, env: Env, path: string): Promise<Response | null> {
  if (path === "/auth/start" && req.method === "POST") {
    if (!env.GOOGLE_CLIENT_ID || !env.GOOGLE_CLIENT_SECRET)
      throw new HttpError(503, "Google sign-in is not configured yet");
    await limit(env, `login:${req.headers.get("cf-connecting-ip") || "local"}`, 20, 600);
    const b = await body(req, 4096);
    if (typeof b.challenge !== "string" || !/^[-_A-Za-z0-9]{43}$/.test(b.challenge))
      throw new HttpError(400, "Invalid desktop challenge");
    const id = random(),
      state = random(),
      verifier = random(),
      nonce = random();
    await env.DB.prepare(
      "INSERT INTO login_attempts(id,state_hash,desktop_challenge,oauth_verifier,nonce,expires_at) VALUES (?,?,?,?,?,?)"
    )
      .bind(id, await hash(state), b.challenge, verifier, nonce, now() + 600)
      .run();
    const url = new URL("https://accounts.google.com/o/oauth2/v2/auth");
    url.search = new URLSearchParams({
      client_id: env.GOOGLE_CLIENT_ID,
      redirect_uri: `${env.PUBLIC_URL}/auth/google/callback`,
      response_type: "code",
      scope: "openid email profile",
      state,
      nonce,
      code_challenge: await challenge(verifier),
      code_challenge_method: "S256",
      prompt: "select_account",
    }).toString();
    return json({ id, authorization_url: url.href });
  }
  if (path === "/auth/google/callback" && req.method === "GET") {
    const url = new URL(req.url),
      state = url.searchParams.get("state");
    if (!state) throw new HttpError(400, "Missing sign-in state");
    const attempt = await env.DB.prepare(
      "UPDATE login_attempts SET status='exchanging' WHERE state_hash=? AND status='pending' AND expires_at>? RETURNING *"
    )
      .bind(await hash(state), now())
      .first<{ id: string; oauth_verifier: string; nonce: string }>();
    if (!attempt) throw new HttpError(400, "Sign-in expired or was already used");
    try {
      if (!env.GOOGLE_CLIENT_ID || !env.GOOGLE_CLIENT_SECRET) throw new Error("configuration");
      const code = url.searchParams.get("code");
      if (url.searchParams.has("error") || !code) throw new Error("declined");
      const response = await fetch("https://oauth2.googleapis.com/token", {
        method: "POST",
        headers: { "Content-Type": "application/x-www-form-urlencoded" },
        body: new URLSearchParams({
          code,
          client_id: env.GOOGLE_CLIENT_ID,
          client_secret: env.GOOGLE_CLIENT_SECRET,
          redirect_uri: `${env.PUBLIC_URL}/auth/google/callback`,
          grant_type: "authorization_code",
          code_verifier: attempt.oauth_verifier,
        }),
        signal: AbortSignal.timeout(15000),
      });
      const tokens = (await response.json()) as { id_token?: unknown };
      if (!response.ok || typeof tokens.id_token !== "string") throw new Error("exchange");
      const profile = await identity(tokens.id_token, env, attempt.nonce);
      const contributor = await env.DB.prepare(
        "INSERT INTO contributors(id,google_sub,name,created_at) VALUES (?,?,?,?) ON CONFLICT(google_sub) DO UPDATE SET name=excluded.name RETURNING id,name"
      )
        .bind(crypto.randomUUID(), profile.sub, profile.name, now())
        .first<Contributor>();
      if (!contributor) throw new Error("profile");
      const ready = await env.DB.prepare(
        "UPDATE login_attempts SET status='ready',contributor_id=?,oauth_verifier='' WHERE id=? AND status='exchanging'"
      )
        .bind(contributor.id, attempt.id)
        .run();
      return message(ready.meta.changes ? "Signed in to Kitor" : "Sign-in was cancelled");
    } catch {
      await env.DB.prepare("UPDATE login_attempts SET status='failed',oauth_verifier='' WHERE id=?")
        .bind(attempt.id)
        .run();
      return message("Sign-in was cancelled or could not be completed");
    }
  }
  if ((path === "/auth/poll" || path === "/auth/cancel") && req.method === "POST") {
    const b = await body(req, 4096);
    if (typeof b.id !== "string" || typeof b.verifier !== "string" || b.verifier.length > 128)
      throw new HttpError(400, "Invalid sign-in attempt");
    const proof = await challenge(b.verifier);
    if (path === "/auth/cancel") {
      await env.DB.prepare("DELETE FROM login_attempts WHERE id=? AND desktop_challenge=?")
        .bind(b.id, proof)
        .run();
      return json({ ok: true });
    }
    const a = await env.DB.prepare(
      "SELECT status FROM login_attempts WHERE id=? AND desktop_challenge=? AND expires_at>?"
    )
      .bind(b.id, proof, now())
      .first<{ status: string }>();
    if (!a) throw new HttpError(401, "Sign-in expired or cancelled");
    if (a.status === "failed") throw new HttpError(401, "Google sign-in failed. Try again");
    if (a.status !== "ready") return json({ pending: true });
    const claimed = await env.DB.prepare(
      "DELETE FROM login_attempts WHERE id=? AND desktop_challenge=? AND status='ready' RETURNING contributor_id"
    )
      .bind(b.id, proof)
      .first<{ contributor_id: string }>();
    if (!claimed) throw new HttpError(409, "Sign-in already completed");
    const token = random();
    await env.DB.prepare(
      "INSERT INTO sessions(token_hash,contributor_id,expires_at) VALUES (?,?,?)"
    )
      .bind(await hash(token), claimed.contributor_id, now() + 7 * 86400)
      .run();
    const contributor = await env.DB.prepare("SELECT id,name FROM contributors WHERE id=?")
      .bind(claimed.contributor_id)
      .first<Contributor>();
    return json({ token, contributor });
  }
  if (path === "/auth/me" && req.method === "GET") return json(await user(req, env));
  if (path === "/auth/logout" && req.method === "POST") {
    const token = req.headers.get("authorization")?.match(/^Bearer ([a-f0-9]{64})$/)?.[1];
    if (!token) throw new HttpError(401, "Missing session");
    await env.DB.prepare("DELETE FROM sessions WHERE token_hash=?")
      .bind(await hash(token))
      .run();
    return json({ ok: true });
  }
  return null;
}

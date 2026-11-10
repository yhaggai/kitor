export interface Env {
  DB: D1Database;
  PUBLIC_URL: string;
  GOOGLE_CLIENT_ID?: string;
  GOOGLE_CLIENT_SECRET?: string;
  GITHUB_APP_ID: string;
  GITHUB_PRIVATE_KEY?: string;
  GITHUB_WEBHOOK_SECRET?: string;
  NOTIFICATIONS_ENABLED?: string;
  RESEND_API_KEY?: string;
  NOTIFICATION_FROM?: string;
}
export type Contributor = { id: string; name: string };
export class HttpError extends Error {
  constructor(
    public status: number,
    message: string
  ) {
    super(message);
  }
}
export const now = () => Math.floor(Date.now() / 1000);
export const random = () =>
  crypto.randomUUID().replaceAll("-", "") + crypto.randomUUID().replaceAll("-", "");
export async function hash(text: string) {
  return [...new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(text)))]
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}
export async function challenge(text: string) {
  return btoa(
    String.fromCharCode(
      ...new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(text)))
    )
  )
    .replaceAll("+", "-")
    .replaceAll("/", "_")
    .replaceAll("=", "");
}
export const json = (value: unknown, status = 200) =>
  Response.json(value, { status, headers: { "Cache-Control": "no-store" } });
export async function body(req: Request, max = 1_000_000): Promise<Record<string, unknown>> {
  const reader = req.body?.getReader();
  if (!reader) throw new HttpError(400, "Missing request body");
  let size = 0;
  const chunks: Uint8Array[] = [];
  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    size += value.length;
    if (size > max) {
      await reader.cancel();
      throw new HttpError(413, "Request too large");
    }
    chunks.push(value);
  }
  const bytes = new Uint8Array(size);
  let at = 0;
  for (const c of chunks) {
    bytes.set(c, at);
    at += c.length;
  }
  try {
    const value: unknown = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(bytes));
    if (!value || typeof value !== "object" || Array.isArray(value))
      throw new Error("object required");
    return value as Record<string, unknown>;
  } catch {
    throw new HttpError(400, "Invalid JSON");
  }
}
export async function limit(env: Env, bucket: string, max: number, seconds: number) {
  const key = `${bucket}:${Math.floor(now() / seconds)}`;
  const row = await env.DB.prepare(
    "INSERT INTO rate_limits(bucket,count,expires_at) VALUES (?,1,?) ON CONFLICT(bucket) DO UPDATE SET count=count+1 RETURNING count"
  )
    .bind(key, now() + seconds)
    .first<{ count: number }>();
  if (!row || row.count > max) throw new HttpError(429, "Too many requests. Try again later.");
}

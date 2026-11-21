import { timingSafeEqual } from "node:crypto";

export async function handleManualCheck(request, token, runCheck) {
  if (request.method !== "POST") return new Response("Method not allowed", { status: 405, headers: { Allow: "POST" } });
  const supplied = new TextEncoder().encode(request.headers.get("authorization") ?? "");
  const expected = new TextEncoder().encode(`Bearer ${token}`);
  if (!token || supplied.length !== expected.length || !timingSafeEqual(supplied, expected)) {
    return new Response("Unauthorized", { status: 401 });
  }
  let action;
  try {
    ({ action } = await request.json());
  } catch {
    return new Response("Invalid request", { status: 400 });
  }
  if (!["scan", "gog", "alerts"].includes(action)) return new Response("Invalid action", { status: 400 });
  await runCheck(action);
  return Response.json({ ok: true, action }, { headers: { "cache-control": "no-store" } });
}

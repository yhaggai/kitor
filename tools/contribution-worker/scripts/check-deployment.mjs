const origin = new URL(process.env.PUBLIC_URL);
if (origin.protocol !== "https:") throw new Error("Health check requires HTTPS");
let healthy = false;
for (let attempt = 0; attempt < 6; attempt++) {
  try {
    const response = await fetch(new URL("/health", origin), {
      redirect: "error",
      signal: AbortSignal.timeout(10000),
    });
    const data = await response.json();
    if (
      response.ok &&
      data.service === "kitor-contribution-worker" &&
      data.configured === true &&
      data.google_configured === true
    ) {
      healthy = true;
      break;
    }
  } catch {
    /* A new deployment may not have propagated yet. */
  }
  await new Promise((resolve) => setTimeout(resolve, 5000));
}
if (!healthy) throw new Error("Deployed Worker failed its configuration health check");
const blocked = await fetch(new URL("/submissions", origin), {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: "{}",
  redirect: "error",
  signal: AbortSignal.timeout(10000),
});
if (blocked.status !== 401) throw new Error("Unsigned submissions must return HTTP 401");
console.log("Worker health and authentication checks passed.");

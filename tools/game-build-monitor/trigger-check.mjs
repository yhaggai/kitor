import { pathToFileURL } from "node:url";

export async function triggerCheck(action, token, transport = fetch) {
  if (!token) throw new Error("Monitor trigger token is not configured");
  if (!["scan", "gog", "alerts"].includes(action)) throw new Error("Invalid monitor action");
  const response = await transport("https://build.kitor.click/admin/check", {
    method: "POST",
    redirect: "error",
    headers: { Authorization: `Bearer ${token}`, "Content-Type": "application/json" },
    body: JSON.stringify({ action }),
    signal: AbortSignal.timeout(180_000),
  });
  if (!response.ok) throw new Error(`Monitor ${action} returned HTTP ${response.status}`);
  const result = await response.json();
  if (result.ok !== true || result.action !== action) throw new Error("Invalid monitor response");
  console.log(`Monitor ${action} completed.`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  await triggerCheck(process.argv[2], process.env.KITOR_MONITOR_TRIGGER_TOKEN);
}

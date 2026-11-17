import { sendEmail } from "../contribution-worker/src/email.ts";
import { applyFileChecks, extractFileChecks, fileChecksKey } from "./file-check-overlay.mjs";

const stores = ["steam", "gog"];
const deliveryKey = "patch-alerts-delivered";

async function hash(value) {
  const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(JSON.stringify(value)));
  return Array.from(new Uint8Array(digest), (byte) => byte.toString(16).padStart(2, "0")).join("");
}

export async function sendPatchAlerts(env, deliver = sendEmail) {
  const recipient = env.PATCH_ALERT_TO;
  if (!recipient || !env.RESEND_API_KEY) return;

  const kv = env.BUILD_SNAPSHOTS;
  const [snapshot, sent, ...checks] = await Promise.all(
    ["latest", deliveryKey, ...stores.map(fileChecksKey)].map((key) => kv.get(key, { type: "json" })),
  );
  if (!snapshot) return;
  const delivered = new Set(sent ?? []);
  const titles = new Map(snapshot.rows.map((row) => [row.gameId, row.title]));
  const entries = [];

  for (const [index, store] of stores.entries()) {
    applyFileChecks(snapshot, checks[index], store);
    for (const { identity, check } of extractFileChecks(snapshot, store)) {
      if (check.status !== "changed" || !check.paths?.length) continue;
      const id = await hash([recipient, identity]);
      if (delivered.has(id)) continue;
      const [gameId, , platform, , patchVersion] = identity;
      entries.push({
        id,
        text: [
          `${titles.get(gameId) ?? gameId} (${store === "steam" ? "Steam" : "GOG"}, ${platform})`,
          `Patch ${patchVersion}. ${store === "steam" ? "Manifest" : "Build"} ${identity.at(-2)} → ${identity.at(-1)}.`,
          ...check.paths.map((path) => `  - ${path}`),
        ].join("\n"),
      });
    }
  }

  if (!entries.length) return;
  entries.sort((a, b) => a.id.localeCompare(b.id));
  const subject = `Kitor patch alerts · ${entries.length} ${entries.length === 1 ? "comparison needs review" : "comparisons need review"}`;
  const body = [
    "The daily build check found files that changed or went missing from builds that Kitor patches depend on.",
    ...entries.map(({ text }) => text),
    "Verify the affected patches before marking the new builds as supported.",
    "https://build.kitor.click/",
  ].join("\n\n");

  await deliver(env, {
    id: `patch-alert/${await hash([recipient, env.NOTIFICATION_FROM, subject, body])}`,
    to: recipient,
    subject,
    text: body,
  });
  await kv.put(deliveryKey, JSON.stringify([...delivered, ...entries.map(({ id }) => id)]));
}

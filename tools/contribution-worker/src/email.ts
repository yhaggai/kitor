import type { Env } from "./shared";

/** The caller must persist delivery state: Resend deduplicates keys for only 24 hours. */
export async function sendEmail(
  env: Pick<Env, "RESEND_API_KEY" | "NOTIFICATION_FROM">,
  message: { id: string; to: string; subject: string; text: string },
  transport: typeof fetch = fetch
): Promise<string> {
  if (!env.RESEND_API_KEY || !env.NOTIFICATION_FROM)
    throw new Error("Email sending is not configured");
  if (!/^[a-zA-Z0-9/_-]{1,256}$/.test(message.id)) throw new Error("Invalid email delivery ID");
  const response = await transport("https://api.resend.com/emails", {
    method: "POST",
    redirect: "manual",
    signal: AbortSignal.timeout(15_000),
    headers: {
      Authorization: `Bearer ${env.RESEND_API_KEY}`,
      "Content-Type": "application/json",
      "Idempotency-Key": message.id,
    },
    body: JSON.stringify({
      from: env.NOTIFICATION_FROM,
      to: [message.to],
      subject: message.subject,
      text: message.text,
    }),
  });
  // Do not surface provider bodies: they can include private recipient details.
  if (!response.ok) throw new Error(`Email provider returned HTTP ${response.status}`);
  const result = (await response.json()) as { id?: unknown };
  if (typeof result.id !== "string" || !result.id)
    throw new Error("Email provider returned no delivery ID");
  return result.id;
}

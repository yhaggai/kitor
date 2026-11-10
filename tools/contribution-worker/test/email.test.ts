import assert from "node:assert/strict";
import { test } from "node:test";
import { sendEmail } from "../src/email";

const config = {
  RESEND_API_KEY: "test-secret",
  NOTIFICATION_FROM: "Kitor <notifications@kitor.click>",
};
const message = {
  id: "approved/submission-1",
  to: "contributor@example.com",
  subject: "Your Kitor contribution was approved",
  text: "Thank you for your correction!",
};

test("Resend sends private recipients with a stable idempotency key and no redirects", async () => {
  const transport = (async (url, init) => {
    assert.equal(url, "https://api.resend.com/emails");
    assert.equal(init?.redirect, "manual");
    const headers = new Headers(init?.headers);
    assert.equal(headers.get("Idempotency-Key"), message.id);
    assert.equal(headers.get("Authorization"), "Bearer test-secret");
    assert.deepEqual(JSON.parse(init?.body as string), {
      from: config.NOTIFICATION_FROM,
      to: [message.to],
      subject: message.subject,
      text: message.text,
    });
    return Response.json({ id: "delivery-1" });
  }) as typeof fetch;
  assert.equal(await sendEmail(config, message, transport), "delivery-1");
});

test("Resend failures do not leak provider response bodies or count as delivery", async () => {
  const transport = (async () =>
    new Response("private recipient", { status: 429 })) as typeof fetch;
  await assert.rejects(
    sendEmail(config, message, transport),
    /^Error: Email provider returned HTTP 429$/
  );
  await assert.rejects(sendEmail({}, message, transport), /not configured/);
  await assert.rejects(
    sendEmail(config, { ...message, id: "bad\nkey" }, transport),
    /Invalid email/
  );
});

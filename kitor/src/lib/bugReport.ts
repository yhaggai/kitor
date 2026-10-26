export const BUG_REPORT_EMAIL = "yhaggai+kitor@gmail.com";
export const MAX_SCREENSHOTS = 5;
export const MAX_SCREENSHOT_BYTES = 10 * 1024 * 1024;
export const SCREENSHOT_TYPES = ["image/png", "image/jpeg", "image/webp"];

export function reportBody(game: string, details: string, version: string, os: string) {
  return `Game / patch: ${game.trim() || "Kitor app"}\n\n${details.trim()}\n\n---\nKitor: ${version}\nSystem: ${os}`;
}

function base64(bytes: Uint8Array): string {
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary);
}

function wrappedBase64(bytes: Uint8Array) {
  return (
    base64(bytes)
      .match(/.{1,76}/g)
      ?.join("\r\n") ?? ""
  );
}

// A portable MIME message preserves attachments without an email service or credentials.
export async function buildEmailDraft(subject: string, body: string, screenshots: File[]) {
  const boundary = `kitor-${crypto.randomUUID()}`;
  const lines = [
    `To: ${BUG_REPORT_EMAIL}`,
    `Subject: =?UTF-8?B?${base64(new TextEncoder().encode(subject))}?=`,
    "X-Unsent: 1",
    "MIME-Version: 1.0",
    `Content-Type: multipart/mixed; boundary="${boundary}"`,
    "",
    `--${boundary}`,
    'Content-Type: text/plain; charset="UTF-8"',
    "Content-Transfer-Encoding: base64",
    "",
    wrappedBase64(new TextEncoder().encode(body)),
  ];
  for (const [index, file] of screenshots.entries()) {
    const extension =
      file.type === "image/jpeg" ? "jpg" : file.type === "image/webp" ? "webp" : "png";
    lines.push(
      `--${boundary}`,
      `Content-Type: ${file.type}`,
      `Content-Disposition: attachment; filename="screenshot-${index + 1}.${extension}"`,
      "Content-Transfer-Encoding: base64",
      "",
      wrappedBase64(new Uint8Array(await file.arrayBuffer()))
    );
  }
  lines.push(`--${boundary}--`, "");
  return lines.join("\r\n");
}

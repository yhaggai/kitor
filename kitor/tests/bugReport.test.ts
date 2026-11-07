import { expect, test } from "bun:test";
import { buildEmailDraft } from "../src/lib/bugReport";

test("email preserves Unicode diagnostics and screenshots as separate MIME attachments", async () => {
  const diagnostics = JSON.stringify({ game: "שלום", error: "missing file" });
  const draft = await buildEmailDraft(
    "Report",
    "Description",
    [new File([new Uint8Array([1, 2, 3])], "private-name.png", { type: "image/png" })],
    diagnostics
  );
  expect(draft).toContain('filename="diagnostics.json"');
  expect(draft).toContain('filename="screenshot-1.png"');
  expect(draft).not.toContain("private-name");
  const attachment = draft
    .split('filename="diagnostics.json"')[1]
    .split("\r\n\r\n")[1]
    .split("\r\n--")[0];
  expect(Buffer.from(attachment.replaceAll("\r\n", ""), "base64").toString()).toBe(diagnostics);
});

test("opting out excludes diagnostics attachment", async () => {
  const draft = await buildEmailDraft("Report", "Description", []);
  expect(draft).not.toContain("diagnostics.json");
});

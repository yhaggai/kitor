import { writeFileSync } from "node:fs";

const values = {};
for (const name of ["GOOGLE_CLIENT_ID", "GOOGLE_CLIENT_SECRET", "BOT_PRIVATE_KEY"]) {
  if (!process.env[name]?.trim()) throw new Error(`Missing ${name}`);
  values[name] = process.env[name];
}
if (!values.BOT_PRIVATE_KEY.includes("-----BEGIN PRIVATE KEY-----")) {
  throw new Error("BOT_PRIVATE_KEY must be PKCS8 PEM");
}
values.GITHUB_PRIVATE_KEY = values.BOT_PRIVATE_KEY;
delete values.BOT_PRIVATE_KEY;
writeFileSync(new URL("../.deploy-secrets.json", import.meta.url), JSON.stringify(values), {
  mode: 0o600,
});

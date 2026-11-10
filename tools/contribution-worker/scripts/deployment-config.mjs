import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

// Keep the checked-in configuration local-only; CI materializes one named environment.
const target = process.env.DEPLOY_ENV;
if (!["staging", "production"].includes(target)) throw new Error("Invalid DEPLOY_ENV");
const required = (name) => {
  const value = process.env[name]?.trim();
  if (!value) throw new Error(`Missing ${name}`);
  return value;
};
const databaseId = required("D1_DATABASE_ID");
if (
  !/^[a-f0-9]{8}(-[a-f0-9]{4}){3}-[a-f0-9]{12}$/i.test(databaseId) ||
  databaseId === "00000000-0000-0000-0000-000000000000"
) {
  throw new Error("D1_DATABASE_ID must identify a provisioned database");
}
const accountId = required("CLOUDFLARE_ACCOUNT_ID");
if (!/^[a-f0-9]{32}$/i.test(accountId)) throw new Error("Invalid CLOUDFLARE_ACCOUNT_ID");
const publicUrl = new URL(required("PUBLIC_URL"));
const name = `kitor-contributions-${target}`;
if (
  publicUrl.protocol !== "https:" ||
  publicUrl.username ||
  publicUrl.password ||
  publicUrl.search ||
  publicUrl.hash ||
  publicUrl.pathname !== "/" ||
  !publicUrl.hostname.startsWith(`${name}.`) ||
  !publicUrl.hostname.endsWith(".workers.dev")
) {
  throw new Error(`PUBLIC_URL must be the HTTPS workers.dev origin for ${name}`);
}
const appId = required("BOT_APP_ID");
if (!/^\d+$/.test(appId)) throw new Error("Invalid BOT_APP_ID");
const root = new URL("../", import.meta.url);
const local = JSON.parse(readFileSync(new URL("wrangler.jsonc", root), "utf8"));
const config = {
  main: "src/index.ts",
  compatibility_date: local.compatibility_date,
  account_id: accountId,
  env: {
    [target]: {
      name,
      workers_dev: true,
      preview_urls: false,
      triggers: local.triggers,
      vars: { PUBLIC_URL: publicUrl.origin, GITHUB_APP_ID: appId },
      d1_databases: [
        {
          binding: "DB",
          database_name: name,
          database_id: databaseId,
          migrations_dir: "migrations",
        },
      ],
    },
  },
};
writeFileSync(new URL("wrangler.deploy.json", root), `${JSON.stringify(config, null, 2)}\n`);
console.log(`Prepared ${target} configuration in ${fileURLToPath(root)}wrangler.deploy.json`);

# Kitor contributions: local Worker + D1

The desktop editor talks to this Worker at `http://127.0.0.1:8787` in debug
builds. Google authenticates contributors; the GitHub App creates draft PRs as
the bot. No contributor GitHub account is needed. Nothing here deploys remotely.

## Google OAuth setup

Use a dedicated Google Cloud project named **Kitor Contributions**. Billing is
not needed for Google sign-in. In Google Auth Platform:

1. Configure branding as **Kitor Contributions**, with the maintainer's support
   and developer contact email. Use External audience for community accounts.
2. Keep the app in Testing while developing. Add the accounts that will test it
   if Google requests a test-user list.
3. Create an OAuth client of type **Web application**, named
   **Kitor Contributions — Local**. The Worker, not the desktop, handles the callback.
4. Add exactly this authorized redirect URI:
   `http://127.0.0.1:8787/auth/google/callback`.
   No JavaScript origin is needed; this is a server-side code exchange.
5. Request only `openid email profile`. No Gmail, Drive, or GitHub access.
6. Store the client ID and secret in local `.dev.vars`. Do not put the secret
   in Vite variables, desktop configuration, source control, or a screenshot.

When hosting later, use a separate production client with the exact HTTPS
Worker callback, set `PUBLIC_URL`, and configure Google's branding/privacy
requirements. Do not change the existing local redirect to a wildcard.

## Local startup

```sh
cd tools/contribution-worker
npm ci
cp .dev.vars.example .dev.vars
chmod 600 .dev.vars
# Fill .dev.vars using the credentials above and the existing bot key.
npm run migrate
npm run dev
```

Wrangler uses local D1; the all-zero database ID is intentionally a local
placeholder. No Cloudflare account or remote D1 database is needed for this POC.
`GET /health` reports configuration flags only, not successful authentication.

The GitHub App is **Kitor Translation Contributions**, ID `5666467`, installed
only on `yhaggai/kitor`, with Contents and Pull requests write permissions.
Its private key stays server-side. `jose` expects PKCS8; if GitHub supplied
`BEGIN RSA PRIVATE KEY`, convert locally with:

```sh
umask 077
openssl pkcs8 -topk8 -nocrypt -in /private/path/bot.pem -out /private/path/bot-pkcs8.pem
```

Put the converted PEM in `.dev.vars` as `GITHUB_PRIVATE_KEY`. Both the config and
converted key must remain restricted local files. Production uses Worker secrets.

## Identity and submission flow

- Kitor generates a secret handoff verifier, sends only its SHA-256 challenge,
  and opens Google in the user's browser. The Worker keeps OAuth state, nonce,
  and PKCE verifier in D1 for ten minutes.
- The callback exchanges the code and verifies Google's signed ID token,
  audience, issuer, expiry, nonce, and verified-email claim. It does not retain
  Google access/refresh tokens or email addresses.
- Only the desktop holding the original handoff verifier can claim the result,
  once. Cancel deletes the attempt; restart creates a new one.
- A seven-day opaque session is kept in OS credential storage, never in the
  webview. D1 stores only its hash. Logout revokes it. Hourly cleanup removes
  expired sessions, login attempts, and rate-limit buckets.
- D1 stores Google subject privately, a random public Kitor contributor ID,
  display name, and submission records. The PR displays name and Kitor ID;
  neither Google subject nor email is published. A Google profile name is
  self-selected, not proof of a person's legal identity.
- The server derives attribution from the verified session and rejects client
  attribution fields. Only Drifter's configured CSV and checksum can change.
  Deterministic per-contributor submission IDs and an atomic D1 lease prevent
  duplicate concurrent work. Retries recover existing branches/PRs.
- Drafts remain local per contributor. An existing anonymous draft is preserved
  in its original folder and is not silently attributed to a new account.

Sign-in is limited to 20 starts per IP per ten minutes and submission to 20
attempts per account per hour. Requests and GitHub responses are bounded.
Browser-origin POSTs are rejected; the native client calls the API directly.

## Validation

```sh
npm run check
npm test
```

Tests use signed test Google tokens and real local D1 (Miniflare), including
identity rejection, protected endpoints, one-time handoff, cancel/restart,
logout, concurrent submission leases, and the complete 9,799-row CSV fixture.
They do not simulate a successful real Google consent or publish test PRs.

Before public deployment: provision real D1 and secrets, set HTTPS URLs and
Google production branding, measure Worker CPU with the full CSV, choose limits
accordingly, and add operational monitoring/account-deletion handling. No patch
build runs inside this Worker; existing repository CI handles patch validation.

## GitHub Actions deployment

`.github/workflows/deploy-contribution-worker.yml` validates PRs without secrets.
Pushes to `v1` affecting the Worker deploy staging. For production, run
**Contribution Worker → Run workflow**, choose branch **v1** and environment
**production**. Other branches cannot run the deployment job. Deployments to the
same environment are serialized and are not cancelled mid-migration.

Before the first deployment, create GitHub environments `staging` and `production`.
Configure **required reviewers** on production and restrict its deployment branches
to `v1`. These protections are repository settings, not enforced by this YAML alone.
If your GitHub plan does not support required reviewers for this repository, establish
an equivalent approval gate before enabling production deployment.

Create two D1 databases once using your Cloudflare account:

```sh
npx wrangler d1 create kitor-contributions-staging
npx wrangler d1 create kitor-contributions-production
```

Record the returned IDs in the corresponding GitHub environment. Never use the
same D1 ID for both environments. Enable your account's workers.dev subdomain.

Each GitHub environment needs these **variables**:

| Variable | Value |
| --- | --- |
| `CLOUDFLARE_ACCOUNT_ID` | Your Cloudflare account ID |
| `D1_DATABASE_ID` | That environment's provisioned database UUID |
| `PUBLIC_URL` | `https://kitor-contributions-staging.<subdomain>.workers.dev` or the production equivalent |
| `BOT_APP_ID` | The contribution bot's app ID (`5666467` for the current bot) |

And these **secrets**:

| Secret | Value |
| --- | --- |
| `CLOUDFLARE_API_TOKEN` | Token scoped to the target account with Workers Scripts Edit and D1 Edit |
| `GOOGLE_CLIENT_ID` | Environment-specific Web OAuth client ID |
| `GOOGLE_CLIENT_SECRET` | Its client secret |
| `BOT_PRIVATE_KEY` | Bot private key in PKCS8 PEM format |

Register each OAuth client's exact `${PUBLIC_URL}/auth/google/callback` URI.
The existing localhost OAuth client stays for local development. Use a sandbox
GitHub repository/App for staging if real test PRs in `yhaggai/kitor` are unwanted;
the current backend still targets that repository in both environments.

CI generates `wrangler.deploy.json` from these values and uses its named environment
for every Wrangler command. The checked-in local configuration is unchanged.
Missing values, placeholder database IDs, and URLs targeting the other environment
fail before migrations. Secrets are written to a mode-600 temporary JSON file,
uploaded with the Worker using `--secrets-file`, and removed even after failure.
They are never included in artifacts or the desktop bundle.

Deployment order: tests → bundle validation → D1 migrations → Worker/secrets →
health and unsigned-submission checks. Migrations must be backward compatible with
the previous Worker because it remains active during migration. A failed deploy or
health check does not roll back already-applied migrations; repair forward or deploy
a compatible prior Worker after inspecting the failure. The health check verifies
configuration and authentication gating, not a full Google login or PR creation.

This workflow deploys only the backend. Desktop packaging continues separately;
set `KITOR_CONTRIBUTION_API_URL` to the appropriate HTTPS URL when building Kitor.

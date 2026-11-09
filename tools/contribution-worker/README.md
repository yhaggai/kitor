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

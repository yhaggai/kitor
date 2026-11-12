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
  Google access/refresh tokens. Verified email addresses are stored privately in
  D1 for transactional contribution notifications.
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

`.github/workflows/deploy-contribution-worker.yml` validates changes on pull requests.
To deploy, run **Contribution Worker → Run workflow** from **v1**, then choose
**staging** or **production**. Each environment serializes its deployments.
Configure the deployment branch restriction to `v1` in both GitHub environments
and set required reviewers for production where supported.

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
| `NOTIFICATIONS_ENABLED` | Set to `true` to enable email delivery |
| `NOTIFICATION_FROM` | Optional sender override for the verified Resend domain |

And these **secrets**:

| Secret | Value |
| --- | --- |
| `CLOUDFLARE_API_TOKEN` | Token scoped to the target account with Workers Scripts Edit and D1 Edit |
| `GOOGLE_CLIENT_ID` | Environment-specific Web OAuth client ID |
| `GOOGLE_CLIENT_SECRET` | Its client secret |
| `BOT_PRIVATE_KEY` | Bot private key in PKCS8 PEM format |
| `GITHUB_WEBHOOK_SECRET` | Shared secret for validating GitHub deliveries |
| `RESEND_API_KEY` | Sending-only key for the verified domain, required when email delivery is enabled |

Set the **repository Actions variable** `KITOR_CONTRIBUTION_API_URL` to the
same HTTPS origin as the production environment's `PUBLIC_URL`. The app
release workflow validates it before creating a draft release and embeds it
in each signed installer. Use a build-time override for staging desktop builds.

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
# Resend notification setup

The transport is in `src/email.ts`; signed merge/release webhooks and the D1
outbox are in `src/notifications.ts`. Sending is disabled unless
`NOTIFICATIONS_ENABLED="true"`.

- Sender: `Kitor <notifications@kitor.click>`.
- Verify `kitor.click` in Resend using the exact records shown in its dashboard.
  Keep the existing Cloudflare inbound MX/SPF records intact. Receiving through
  Resend is not required.
- Create a **Sending access** API key restricted to `kitor.click` and put
  `RESEND_API_KEY` and `NOTIFICATION_FROM` in the ignored, mode-600 `.dev.vars`
  for local development. Never put the key in desktop configuration or Git.
- For hosted environments use separate keys and Wrangler secrets.
- The free plan allows 100 emails/day and 3,000/month. The persistent D1 outbox
  sends at most four messages per hourly tick and reserves at most 90 attempts/day.
  Resend's idempotency keys only deduplicate for 24 hours; uncertain deliveries
  must not be blindly retried beyond that window.
- The Google-verified email is stored privately after sign-in and never returned
  in the public contributor profile or PR. Existing users must sign in again.


## Activate merge and release emails

1. Apply migration `0002_notifications.sql` locally or to the intended deployed D1.
2. Set `GITHUB_WEBHOOK_SECRET` to a cryptographically random secret. Keep the
   existing Resend key private. Configure `NOTIFICATION_FROM` and explicitly set
   `NOTIFICATIONS_ENABLED=true` only in the intended sending environment.
3. Deploy to a public HTTPS Worker URL. In the GitHub App webhook configuration,
   use `https://<worker>/webhooks/github`, the matching secret, and subscribe to
   **Pull request** and **Release** events. Localhost cannot receive GitHub events.
   App release subscriptions may need repository Contents read permission (the
   contribution bot already has Contents write). Do not change other permissions.
4. Redeliver a signed test event from GitHub and inspect its response. Failed
   GitHub deliveries need manual redelivery; GitHub does not automatically retry.
   Nothing in local tests merges or publishes the POC PRs.

Only merged PRs into `yhaggai/kitor:v1` with a matching saved submission/PR URL
are queued. Closing without merging does nothing. Duplicate events reuse the
same outbox ID. Emails contain a PR/release link, never OAuth tokens. Existing
sessions without stored email wait in the queue until the next Google sign-in.

The hourly cron sends queued emails. Release events may arrive before merge
notifications: they remain in D1 until a matching submission is available.
One submission/release pair is checked per tick, with negative checks revisited
after a day. Release checks require published non-prerelease assets, the expected
manifest filename/version/checksum, merge ancestry, and matching corrected CSV
values at the tag. A reverted correction does not generate a release email.
Release emails wait for the approval email to be sent first.

Delivery attempts freeze the recipient, sender, subject and body for Resend's
24-hour idempotency window. After 23 hours, ambiguous failures move to
`needs_review` rather than risk duplicate mail; inspect the provider log before
resolving them. Monitor pending/needs_review counts. The 90/day budget is for this
Worker only; other applications sharing the Resend account consume its quota too.
This is transactional contribution mail, not a marketing subscription.

## Optional email consent

The submission review includes an unchecked email checkbox. Missing consent
(including older app versions and existing submissions) defaults to no email.
The server stores the per-submission choice and opt-in timestamp, and checks
consent again immediately before provider dispatch. Resend is disclosed in the
review UI. No email preference or email address is published in the PR.

Each notification includes a private, random unsubscribe link. GET shows a
confirmation; POST stops all contribution emails and cancels pending/review
items without requiring login. A message already in flight may still arrive.
A new, explicitly checked submission can resume emails for that submission;
previous submissions stay opted out. Do not log unsubscribe query tokens.
Apply `0003_email_consent.sql` before running the updated backend.

## Staging deployment verified 2026-11-11

- Worker: `https://kitor-contributions-staging.yhaggai.workers.dev`
- D1: `kitor-contributions-staging` (`28a562f7-aaa9-4e80-a0cd-70b4d86a09bd`).
- Repository webhook: `710452705`, signed JSON, `pull_request` and `release`.
- Google staging callback registered alongside the existing localhost callback.
- Final schedule: hourly; merge target defaults to `v1`. `NOTIFICATION_BASE_BRANCH`
  can temporarily isolate staging tests; it is not set in the final deployment.
- Controlled PR #30 merged only into `codex/notification-e2e-base`, never `v1`.
  GitHub delivered HTTP 200; the scheduled Worker sent exactly one approval email,
  and Resend reported Delivered. Replaying the merge did not duplicate it.
- The public unsubscribe endpoint cancelled a queued synthetic release follow-up
  with zero delivery attempts. No patch release was published for this test;
  tag/CSV inclusion checks remain covered by local tests.
- The Contribution POC app was rebuilt with `KITOR_CONTRIBUTION_API_URL` pointing
  to staging. Hosted sign-in uses its own keyring entry; sign in again in the app.

The deployment was performed explicitly from the contribution branch. This does
not configure GitHub Actions environment secrets or enable automatic deployments.
Production deployment remains separate. The isolated test account is unsubscribed.

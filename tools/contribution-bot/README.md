# Legacy local Kitor contribution bot

The desktop now uses the authenticated [Worker + D1 service](../contribution-worker/README.md).
This earlier prototype is retained for reference and is not the active endpoint.

Run from the repository root:

```sh
cargo test --manifest-path tools/contribution-bot/Cargo.toml
KITOR_BOT_PRIVATE_KEY_FILE=/absolute/path/outside/repository/bot.pem \
  cargo run --manifest-path tools/contribution-bot/Cargo.toml
```

Install **Kitor Translation Contributions** (App ID 5666467) on only
`yhaggai/kitor`, then generate its private key in GitHub App settings. Keep the
key in a directory accessible only to your user (directory mode 700, file mode
600). Never commit it, ship it with Kitor or paste it into logs. The environment
variable contains a file path, not the key itself.

Without the variable the service starts unconfigured for local UI testing.
`GET http://127.0.0.1:47831/health` reports configuration, not verified GitHub access.
Open Drifter's editor, review saved corrections and select **Check connection**,
then **Send corrections**. The response links to a draft PR authored by the bot.
No contributor GitHub login, fork or local clone is needed.

The server accepts only Drifter corrections, validates canonical source rows and
placeholders, checks the preview digest, and updates CSV plus manifest checksum.
PRs do not bump release versions and are marked POC / do not merge.

This is strictly a local prototype, bound to 127.0.0.1. Browser-origin requests
are rejected and request sizes are bounded. The custom client header is not an
authentication secret. Do not expose this service through a proxy or tunnel;
a hosted version needs contributor authentication, abuse controls and rate limits.

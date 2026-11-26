# Kitor landing page

Hebrew RTL Astro website with React islands reusing the desktop GameBox component.

Use Node.js 24 for local development and builds.

```sh
cd website
npm ci
npm run dev
npm run check
npm run build
```

The landing page is `/` and the catalog is `/games/`. Every game in `kitor/patches/registry.json` gets static pages at `/games/<id>/` and `/en/games/<id>/` on the next website build. Use a stable game ID and add the artwork and any available descriptions or screenshots to the shared desktop data. There is no separate website entry or page to create. A record added only to `patches-metadata/` is a catalog candidate until it is added to the app registry.

`npm run sync` runs before development and builds. It projects the app registry and shared artwork, descriptions, and screenshots into `src/generated/catalog.json` and copies the shared desktop GameBox component. Edit the desktop sources and regenerate the checked-in snapshot for standalone hosting builds.

`npm run build` checks that every registered game has both static pages, a download button, a link from each catalog, and both sitemap entries. The website validation workflow runs this build on pull requests that change the website or its shared inputs. The generated pages become live when the updated website is deployed.

The only desktop component addition is an optional `locale` prop for Hebrew viewer controls; English remains the default. The website provides the pure browser `cn` utility so the native Tauri API is not bundled.

Catalog search accepts English and Hebrew titles. The landing page recommends a desktop installer using browser platform and architecture hints. It selects the highest published stable `app-vX.Y.Z` GitHub release that contains all three installers. The download section always shows all builds and falls back to verified installer links if the API is unavailable. Update the fallback version in `src/lib/downloads.ts` after publishing a new stable desktop release.

Generated night scenery is in `public/art`; source game art lives in the ignored local `assets/` workspace. Edge characters are lightweight SVG pixel illustrations, animated with IntersectionObserver and disabled motion for reduced-motion preferences.

## Cloudflare Pages deployment

The **Deploy website** GitHub Action checks and builds the website, verifies the generated game pages, and uploads `website/dist` with Wrangler. It runs on pushes to `v1` that change the website or its shared inputs, and can also be started manually from Actions on `v1`.

Create a Cloudflare Pages Direct Upload project and set its production branch to `v1`. In GitHub **Settings → Secrets and variables → Actions**, configure these repository variables:

| Variable | Value |
| --- | --- |
| `CLOUDFLARE_ACCOUNT_ID` | Cloudflare account ID containing the Pages project |
| `CLOUDFLARE_PAGES_PROJECT` | Exact Pages project name |
| `KITOR_SITE_URL` | Public HTTPS origin, such as `https://kitor.click` or the project's `https://<project>.pages.dev` URL |
| `INFISICAL_PROJECT_SLUG` | Infisical project containing the Pages token |
| `INFISICAL_IDENTITY_ID` | Infisical machine identity for website deployment |

Store `CLOUDFLARE_PAGES_API_TOKEN` at the root of the Infisical `prod` environment. Scope the token to the Pages account with **Account → Cloudflare Pages → Edit** permission. Grant the machine identity read access to the token and configure GitHub OIDC trust for the `v1` branch and `.github/workflows/deploy-website.yml`. The workflow fetches the token immediately before deployment.

Set the Pages production branch to `v1`. If the project already uses Cloudflare Git integration, disable its automatic builds when using this Action to avoid duplicate deployments.

Merge the workflow into `v1`, then select **Actions → Deploy website → Run workflow → v1** for the first deployment. Custom domains must also be attached to the Pages project in Cloudflare. This Action deploys to your Cloudflare account; it does not update the separate ChatGPT Sites publication recorded in `.openai/hosting.json`.

Cloudflare documents this flow in [Use Direct Upload with continuous integration](https://developers.cloudflare.com/pages/how-to/use-direct-upload-with-continuous-integration/).

## Store search prototype

`/store-search/` is a separate, unlisted research page. Select a catalog game to fetch current offers from CheapShark, grouped by active store and sorted by price. `?game=undertale` preselects a game. Steam IDs provide exact identity matches; title searches ask the user to confirm the edition. Missing offers describe missing provider coverage, not verified store unavailability. Prices are USD and region/patch compatibility must be checked separately.

The browser calls CheapShark directly with a 15-second timeout and a five-minute in-memory cache. No API key or backend is required. CheapShark requires its redirect links and prohibits using your own affiliate links for tracked stores; this prototype does not monetize purchases. Direct retailer feeds or separately agreed terms are needed before building Kitor affiliate monetization on this feature. See https://apidocs.cheapshark.com/.

Selecting a game also exposes a G2A marketplace search link using its catalog title. This link is independent of CheapShark success and opens G2A in a new tab; no G2A availability or prices are inferred. Inline G2A offers would require an authenticated integration with its Export API.

Run the focused service tests with Node 24 or newer: `node --test website/scripts/store-search.test.ts` from the repository root.

## Website analytics

Production builds use the Kitor GA4 web stream (`G-1EVLL5CNSE`). Set `PUBLIC_GA_MEASUREMENT_ID` to override the stream ID or set it to an empty value to disable analytics for a build. Development skips the Google tag.

The Hebrew and English landing pages send `installer_download` when a visitor clicks a direct installer link. The `installer_platform` parameter is `windows-x86_64`, `darwin-aarch64`, or `darwin-x86_64`. The `page_language` parameter is `he` or `en`.

In GA4 Admin, create event-scoped custom dimensions named **Installer platform** and **Page language**, using the corresponding parameter names above. In Explore, filter Event name to `installer_download` and break down Event count by Installer platform. Add Page language for the Hebrew and English split. For a deployment smoke test, click a direct installer link on each landing page and check the event and parameters in GA4 Realtime.

## Game build monitor

`/build-monitor/` displays the public Steam app build and GOG Galaxy builds for store IDs in the patch registry. The GitHub Actions workflow **Monitor patched game builds** runs daily and supports manual dispatch. It publishes `game-builds.json` to the `game-build-monitor` branch. Forks and local previews can set `PUBLIC_GAME_BUILD_SNAPSHOT_URL` to another snapshot URL.

Add a documented store build ID or an exact Steam depot manifest to the patch release’s `supported_builds` array in `kitor/patches/registry.json`. Include the store, platform, and evidence path. Use `released_on` and `release_date_source` when the store release date is known. Use `first_seen_on` if the source only identifies the first observation of a depot manifest. Latest Steam dates use the public branch update timestamp, bounded by the app’s initial Steam release when available. GOG dates use the public Galaxy build publication timestamp. The first successful scan records what the store serves. Subsequent scans retain the last observed change and compare the latest build or specific depot against the documented baseline. A build difference calls for a patch check. Store lookup failures retain the last successful value. GOG offline installers and store listings without IDs in the registry are outside this scan.

The production monitor is at [build.kitor.click](https://build.kitor.click). Its Cloudflare Worker serves the dashboard and its same-origin `/game-builds.json` endpoint. Cloudflare Cron runs the same scanner daily at 05:23 UTC and persists the results in Workers KV. Successful patch publication and every published release refresh the KV registry and scan production automatically through **Refresh build dashboard**. Patch versions and supported builds no longer require a Worker deployment. Redeploy when adding game cards or changing the dashboard UI.

Run these from the repository root to deploy the dashboard and registry:

```sh
node tools/game-build-monitor/prepare-cloudflare.mjs
wrangler deploy --config tools/game-build-monitor/wrangler.jsonc
```

The production build links game pages back to the main Kitor website. Set `KITOR_SITE_URL` when building if the main site URL changes.

### Steam manifest file checks

The scanner can inspect changed Steam depots on a machine with the native Steam helper. Build the helper and set its path when running a scan:

```sh
cargo build --locked --manifest-path kitor/native-steam/Cargo.toml
export KITOR_STEAM_MANIFEST_BIN="$PWD/kitor/native-steam/target/debug/kitor-steam"
node tools/game-build-monitor/monitor.mjs next-game-builds.json previous-game-builds.json
```

Omit the previous snapshot argument on the first run. The helper uses Steam's local depot cache when possible. Set `KITOR_STEAM_DEPOT_CACHE` to override the macOS default. For uncached manifests, run the helper's `connect` command once to authenticate an account with access to the depots. The helper requests only manifests and prints per-file SHA-1 values. It does not download Steam content chunks.

For each changed depot with a recorded manifest baseline and a matching patch plan, the scanner checks the original files the patch depends on. It records unchanged inputs, changed or missing inputs, or an unavailable comparison. A changed input calls for patch verification. Successful checks are reused for the same manifest pair and patch file list. App-only baselines have no depot manifest to compare. The Cloudflare scan continues to fetch public metadata.

The production Worker compares GOG Galaxy manifests in Cloudflare once a day at 05:43 UTC, after the public scan at 05:23 UTC. Deployment bundles the patch plans for recorded GOG baselines. The check reads English and language-neutral base-game manifests and compares file hashes when the formats are comparable. GOG offline installers are outside this check.

The hosted Steam workflow `Check Steam patch files` runs at 06:03 UTC when enabled. Set the repository variables `KITOR_STEAM_CLOUD_ENABLED=true` and `KITOR_MONITOR_CLOUDFLARE_ACCOUNT_ID`. Store a Steam session for an account with depot access in the GitHub secret `KITOR_STEAM_CLOUD_SESSION` as a JSON object with `account`, `steam_id` (integer), and `refresh_token`. Store a Cloudflare token with Workers KV write access in `KITOR_MONITOR_CLOUDFLARE_TOKEN`. The hosted runner publishes results for the exact manifest pair in the live dashboard snapshot.

The Cloudflare Worker sends the daily patch alert at 06:33 UTC to `notifications@kitor.click` when a current manifest comparison shows a patch input changed or is missing. The alert lists each affected patch and its files. The Worker records successful alerts in KV to avoid repeating them. Provision a sending-only `RESEND_API_KEY` on the build monitor Worker to enable delivery.

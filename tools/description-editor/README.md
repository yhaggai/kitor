# Kitor description editor

From the repository root, use Bun and an authenticated GitHub CLI with permission to push branches and open pull requests in `yhaggai/kitor`.

```sh
gh auth status --hostname github.com
bun tools/description-editor/server.ts
```

Open <http://127.0.0.1:4732>. Search for a game, edit its Hebrew description, and select **Open pull request**. The editor loads the current descriptions from GitHub's `v1` branch. It uses your GitHub CLI credentials on the local server to create a branch and a PR against `v1`. It edits the selected description in `kitor/src/data/game-descriptions.he.json` and leaves your working tree alone. The PR runs the repository's catalog checks. If someone merges a catalog change while you are editing, copy your draft and reload the editor before submitting.

## Publishing the feed

Set repository variable `CLOUDFLARE_ACCOUNT_ID` and repository secret `KITOR_DESCRIPTION_R2_TOKEN` with Cloudflare Workers R2 Storage edit access to the account that owns `kitor-assets`. Merge the workflow into `v1`, then run **Publish game descriptions** from the Actions tab if the initial push has not already published the feed. The workflow validates description changes on PRs and uploads the latest `v1` catalog on relevant merges. The public feed is served at <https://assets.kitor.click/catalog/game-descriptions-v1.json>.

Desktop builds with feed support check on startup and every 15 minutes while open. A successful response updates the visible description and the local cache. Offline, the app uses the last valid cache, then the bundled description for games missing from it. Reverting a description commit and merging the revert republishes the previous text. The website continues to pick up the same source file through its static build.

To validate the descriptions locally, run `node kitor/tools/build-description-feed.ts` with Node 24 or newer.

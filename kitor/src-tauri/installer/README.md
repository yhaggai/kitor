# Kitor installers and app updates

## Build locally

From `kitor/`:

- macOS installer: `bun run tauri build --bundles dmg -- --locked`
- Windows setup: `bun run tauri build --bundles nsis -- --locked`

Outputs are under `src-tauri/target/release/bundle/`. Local builds do not need the update-signing key. To generate signed update artifacts as well, set `TAURI_SIGNING_PRIVATE_KEY` to the private key's path or content, set `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (an empty string for an unencrypted key), and add `--config src-tauri/tauri.release.conf.json` before `-- --locked`.

The Mac installer has a branded drag-to-Applications window. Windows uses an NSIS wizard with English/Hebrew selection and per-user installation. Artwork is committed; regenerate it on macOS with Pillow and `python3 installer/generate-assets.py` from `src-tauri/`.

## Update behavior

Kitor quietly checks once each time it starts. A new version opens a dialog with release notes, **Later**, and **Update and restart**. Settings also has **Check for updates**. An offline startup does not interrupt the user; manual checks explain failures and allow retrying.

Tauri verifies each downloaded update with the embedded public key before installation. An invalid signature never reaches the installer. Development builds can check but cannot install updates. Installation is blocked while a game download, patch installation/restore, or ScummVM installation is running. Updates retain the app identifier and data directory, library database, settings, and installed patches.

The public endpoint is:

`https://github.com/yhaggai/kitor/releases/download/app-updates/latest.json`

`app-updates` is a dedicated metadata release. Game-patch releases and GitHub's general “Latest release” selection do not affect app updates. Users do not need a GitHub token. The feed first becomes available when the first complete app release is published; until then checks report that they are unavailable.

## Release a version

1. Set the same stable `X.Y.Z` version in `kitor/package.json`, `kitor/src-tauri/Cargo.toml`, and `kitor/src-tauri/tauri.conf.json`; refresh and commit Cargo.lock.
2. Merge to `v1`, then run **Release Kitor app** from that branch.
3. The workflow creates a draft `app-vX.Y.Z` release, builds macOS Apple Silicon, macOS Intel, and Windows x64 installers, and uploads signed update artifacts. All three platforms must succeed before the draft's manifest is generated. A failed build leaves the release as a draft. Retry failed jobs in the original run; a new run refuses to overwrite an existing release.
4. After the workflow succeeds, replace the draft's placeholder release notes and review the installers. Publish the draft using your GitHub account. (Publishing with the workflow's `GITHUB_TOKEN` does not trigger subsequent workflows.)
5. The published-release workflow validates the complete set of signed update assets and advances `app-updates/latest.json`. Its version check prevents moving the feed backward. Release notes are read at publication time.

Do not publish an incomplete draft. The feed workflow refuses incomplete releases, but GitHub itself still allows publishing them. Assets are hosted as release downloads instead of Actions artifacts. The separate **Build Kitor installers** PR workflow checks both operating systems without signing secrets; optional preview-artifact uploads are off by default.

## Signing key

`TAURI_SIGNING_PRIVATE_KEY` is a GitHub Actions repository secret. `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` is optional for password-protected keys. The committed Tauri config contains only the matching public key. Keep a secure backup of the private key; generating a replacement would prevent existing installations from verifying future updates.

These update signatures are separate from Apple Developer ID/notarization and Windows Authenticode signing. The workflows currently provide Tauri update signatures; platform signing/notarization still needs the corresponding certificates before broad public distribution.

## Validation

From the repository root:

```sh
bun test kitor/tests/appUpdates.test.ts
python3 -m unittest discover -s kitor/tools -p test_app_release.py
python3 kitor/tools/app_release.py validate
cargo test --locked --manifest-path kitor/src-tauri/Cargo.toml --lib services::app_updates::tests
bun run --cwd kitor build
```

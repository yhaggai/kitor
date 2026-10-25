# Kitor - ScummVM Translation Patch Manager

A desktop application built with Tauri + React for managing ScummVM games and community translation patches.

## Features

- 🎮 **Game Library** - Automatically scan and detect ScummVM games
- 🌍 **Translation Patches** - Browse and install community translation patches
- 📦 **One-Click Install** - Download, backup, and apply patches automatically
- 🔄 **Backup & Restore** - Back up only files affected by a patch
- ⚡ **Fast & Lightweight** - Built with Tauri (3-5 MB binary)
- 🎨 **Modern UI** - Beautiful interface with shadcn/ui components

## Prerequisites

Before running Kitor, you need to install:

1. **Rust** - Required for Tauri backend
   ```bash
   # Windows (PowerShell)
   winget install Rustlang.Rustup
   
   # Or download from: https://rustup.rs/
   ```

2. **Bun** - JavaScript runtime and package manager
   ```bash
   # Already installed at: C:\Users\haggai\.bun\bin\bun.exe
   # Add to PATH or restart terminal
   ```

3. **ScummVM** - Required to run games
   - Download from: https://www.scummvm.org/downloads/
   - Install and Kitor will auto-detect it

## Development Setup

1. **Install Rust dependencies**:
   ```bash
   cd kitor/src-tauri
   cargo check
   ```

2. **Install frontend dependencies** (already done):
   ```bash
   cd kitor
   bun install
   ```

3. **Run development server**:
   ```bash
   bun run tauri dev
   ```

## Building for Production

```bash
bun run tauri build
```

The compiled application will be in `src-tauri/target/release/`.

## Project Structure

```
kitor/
├── src/                      # React frontend
│   ├── components/           # UI components
│   │   ├── ui/              # shadcn/ui components
│   │   ├── GameLibrary.tsx
│   │   ├── GameCard.tsx
│   │   ├── PatchBrowser.tsx
│   │   └── Settings.tsx
│   ├── store/               # Zustand state management
│   ├── lib/                 # Utilities
│   └── App.tsx              # Main app component
├── src-tauri/               # Rust backend
│   ├── src/
│   │   ├── commands/        # Tauri commands (API)
│   │   ├── services/        # Business logic
│   │   ├── models/          # SeaORM entities
│   │   ├── db.rs            # Database setup
│   │   └── lib.rs           # Main entry point
│   └── Cargo.toml
└── patches/
    └── registry.json        # Patch metadata
```

## Tech Stack

**Frontend:**
- React + TypeScript
- Vite (build tool)
- TailwindCSS + shadcn/ui
- Zustand (state management)
- Lucide Icons

**Backend:**
- Tauri 2.x
- Rust
- SeaORM (SQLite ORM)
- tokio (async runtime)
- reqwest (HTTP client)
- zip (archive extraction)

## Usage

### Adding Games

1. Open "Scan for Games" in the Library tab and select "Choose a folder…"
2. Select a folder containing ScummVM games
3. Kitor will automatically detect compatible games

You can also select "Automatically find Steam & GOG games" from the scan menu.

### Kyrandia on macOS

Select the GOG `The Legend of Kyrandia(Book One).app` bundle, its parent folder
when scanning, or the game data folder directly. Kitor resolves
`Contents/Resources/game/game` for patching and launching; flat installations
and Windows `DAT` folders are also supported.

Some GOG bundles contain Book Two's `FATE.PAK` as well. Kitor copies Book One's
`GEMCUT.PAK` into a small launch folder under its app data directory and loads
other assets with ScummVM's `--extrapath`, avoiding a false Book Two match without
removing game files. ScummVM detects English or Hebrew from the current files.

Regression tests: run `cargo test --lib` from `src-tauri`. To also check a local
installation, set `KITOR_KYRANDIA_APP` to the app bundle path and run
`cargo test --lib -- --include-ignored`.

### Installing Patches

1. Go to the Patches tab
2. Browse available translation patches
3. Click "Install Patch" to download and apply
4. Only affected existing files are backed up; newly created files are tracked for uninstall

Private GitHub release assets require GitHub CLI (`gh`) on Kitor's PATH and an
account signed in with `gh auth login` that can access the repository. Kitor
tries the public URL first and uses the existing CLI login if GitHub returns
404 for a release asset. Public downloads need no GitHub login. Failed downloads
are discarded, and ZIP contents are validated before installation begins.

### Launching Games

Click the "Play" button on any game card to launch it through ScummVM.

## Patch file-change manifests

Each release ZIP includes `patch.json` from `patches-workspace/<game>/he/`.
The catalog references it with `install_manifest: "patch.json"`; the package's
game, language and version must match the catalog. Current package versions use
`-kitor2` so existing releases without manifests cannot be mistaken for these
packages. Publish the new releases before using the updated catalog downloads.

A minimal package manifest looks like:

```json
{
  "schema_version": 1,
  "scummvm_id": "example",
  "language": "he",
  "version": "1.0-kitor2",
  "filename": "example.zip",
  "platforms": { "windows": {}, "osx": {} },
  "files": [
    {
      "path": "dialogue.dat",
      "operation": "install",
      "source": {
        "kind": "copy",
        "path": "translated/dialogue.dat",
        "sha256": "<SHA-256 of the packaged file>"
      }
    }
  ]
}
```

`files` is a shared list; `platforms.<platform>.files` overrides it when platform
archives differ, as in Undertale. Every target path is relative to the detected
game data directory. No wildcards, absolute paths, traversal or symlinks are
accepted. Case-insensitive matches preserve the existing filename's spelling.

Operations:
- `install`: replace an existing file or add it if absent, recording which occurred.
- `modify`: require and replace an existing file.
- `create`: require an absent target; an existing file is a conflict.
- `delete`: require and remove an existing file, with no `source`.

Sources are `copy`, `bsdiff`, or `xdelta`. Each specifies the packaged asset path
and SHA-256. Delta sources also declare an `input` path relative to the game.
`original_sha256` and `installed_sha256` pin supported input and output versions;
Undertale requires both. Xdelta validates the source through its delta decoder.
Windows xdelta tools are bundled and checksum-verified; macOS requires `xdelta3`
on PATH. Kitor does not auto-install dependencies or run package install scripts.
BSDIFF uses Kitor's decoder with Python 3 (`py -3` on Windows).

All outputs are generated in temporary staging before game files change. Only
manifest outputs are installed: package scripts, source catalogs, tools, and
legacy uninstallers stay out of the game folder. Space Quest's loose resources
are mapped from `sq1/PATCHES/` into `PATCHES/` inside the detected SCI game
data directory so ScummVM can recognize the Hebrew translation. For SQ1 VGA
1.0-kitor3, uninstall the older flattened patch through Kitor before installing
the replacement; do not manually delete loose resources or their backup journal.

Kitor stores selective originals and a durable transaction manifest under its
app data directory's `patch-transactions/<installation-hash>/<transaction>/`.
The manifest records each target's original and installed hash, whether it
originally existed, and operation progress. Original file permissions are
preserved. Retrying an installed version reuses its original backups; overlapping
patches/upgrades require uninstall first. Interrupted transactions are recovered
on the next installation attempt. Recovery data is retained if a conflict stops
rollback, and after restoration for diagnosis/retry.

Uninstall restores only changed originals and removes only patch-created files
and empty patch-created directories. Unrelated files, saves, settings, and files
added later remain untouched. Changed targets or missing/corrupt backups stop
restoration with an error. Existing full-folder backups are preserved but cannot
be automatically restored because they lack reliable file ownership metadata.

Validation: `cargo test --lib` from `src-tauri`. The optional
`real_undertale_archive_roundtrip_on_disposable_copy` test uses
`KITOR_UNDERTALE_ARCHIVE` pointing to an original `game.ios` or `data.win`; it
copies that file to a temporary installation. Run it with `-- --ignored`.
Synthetic Windows/macOS layout tests are not native Windows runtime validation.

## Database

Kitor uses SQLite to store:
- Game library (titles, paths, metadata)
- Patch information (versions, download URLs)
- Installation history (backups, status)

Database location: `%APPDATA%/kitor/kitor.db`

## License

MIT

## Contributing

Contributions welcome! Please feel free to submit a Pull Request.

### UNDERTALE Hebrew (Alpha 2)

The catalog includes the Steam macOS and Windows patch. Add `UNDERTALE.app`,
its containing folder, or the Windows game folder. Kitor detects the archive,
backs up only the game archive, applies the verified delta, and launches the native
game. Python 3.10+ is required (`python3` on macOS; `py -3` on Windows).
Save and close the game before installing or restoring. Keep English selected.
Only the original archive hashes in the bundled manifests are supported.
The translation is a draft; upstream Windows runtime testing is still pending.

The patch binaries and full build sources are checked in under
`patches-workspace/undertale/he/`, with provenance in `KITOR.md`. The existing
Deploy Patches workflow publishes the catalog ZIP when pushed to main or v1.
The download URL becomes available after that release is published.

## Steam ownership integration

In **Settings → Steam Integration**, connect your account in your system browser,
then enter your own [Steam Web API key](https://steamcommunity.com/dev/apikey).
Steam OpenID verifies your Steam ID; it does not grant access to private game details.
Use the Steam privacy-settings link if library lookup reports unknown ownership.

The API key is saved in the system credential store (macOS Keychain, Windows
Credential Manager, or Linux Secret Service) and restored on startup. It is sent
only to Steam's HTTPS API and never stored in app files or returned in status
responses. Credential-store failures are shown explicitly; there is no plaintext
fallback. The verified Steam ID is remembered in `steam-account.txt` in Kitor's
app-data directory. Disconnect removes the saved key, ID, and cached ownership,
and cancels pending login. Keys entered before secure persistence was added need
to be entered once more because the earlier version kept them only in memory.

Owned games open their Steam library entry, regardless of local installation.
Other games, private libraries, and lookup errors open the store page. Missing or
failed responses never establish that a user does not own a game. Lookup is refreshed
on demand when clicking a Steam button after five minutes, or with Refresh library.
Steam's active client account should match the account connected in Kitor.

Validation: `cargo test --lib` from `src-tauri`, and `npm run build` from `kitor`.
For a live smoke test, connect a Steam account, enter its API key, check an owned
but uninstalled title, then disconnect and verify the button returns to the store.
Also check a private game-details profile: it should report unknown ownership.

### Steam Windows downloads on macOS (pilot)

Monkey Island 2 has a single **Steam** button. On macOS it opens a Windows-version
explanation; **Next** starts the download and shows an animated, indeterminate
progress bar. The dialog can be hidden and reopened while downloading. Steam
must be installed, initialized, online, and signed into an account owning the game;
Kitor does not require a Steam Web API key for downloads. The backend invokes the
local Steam client with `-console +download_depot 32460 32461`, monitors only new
console log entries, and imports verified game files into Kitor's application data
`games/monkey2-se-steam` folder. It then registers the game in the library, where
users can install the Hebrew patch and launch through ScummVM.

This curated pilot supports only app 32460/depot 32461 on macOS. Steam may display
its console or request sign-in. Keep Kitor open during download/import. Progress
is indeterminate; failures and a one-hour timeout are reported. A timeout does not
cancel Steam's own download. Existing registered installations are reused, never
replaced, and existing destination directories are not overwritten. If registration
fails after import, use Add Game with the managed folder. Steam console formatting
is an unofficial integration surface and may change between client releases.

Validation: `cargo test steam_download_service --lib`. An opt-in live test downloads
using the current Steam account and imports into a temporary library:
`cargo test steam_download_and_register --lib -- --ignored --nocapture`.


### Space Quest Collection on macOS

The Steam buttons for SQ1 EGA (`sq1`), SQ1 VGA (`sq1sci`), SQ2, SQ3,
and SQ5 share app **10110**, depot **10111**. The dialog defaults to the clicked
game and offers the other supported editions. Downloads/imports share one task;
hiding the dialog does not cancel Steam. Timeout retries resume the recorded
request rather than issue another download.

Verified on 2026-10-25 using the signed-in Steam client: public manifest
`6725209980292758062`, 644,216,240 download bytes. ScummVM **2026.1.0** detected
and displayed an opening screen for all five editions. SQ1 EGA is DOS 2.2;
SQ2 is DOS 2.0F. SQ1 EGA also yields a CoCo3 detection, so collection launches
select the DOS platform and explicit engine/game identity. Unpatched games do
not inherit the registry's Hebrew language restriction; active patches retain it.
These are original-game smoke tests, not full playthroughs or Hebrew-patch validation.

`src-tauri/data/space_quest_steam.json` records the exact source directories,
allowlisted files, and SHA-256 digests from that verified Steam manifest.
The importer stages and verifies independent folders under `games/<id>-steam`
and retains pristine shared sources under `steam-sources/10110-6725209980292758062`.
It reuses matching Steam files even without an installed-game appmanifest,
rejects unknown bytes and symlinks, and preserves registered installations,
saves, and patch state. A Steam update with changed files requires revalidation
of these recipes. Games outside the five catalog editions are not imported.

Validation commands (from `kitor/`):

```sh
npm run build
cargo test --manifest-path src-tauri/Cargo.toml --lib
# Opt-in disposable integration test; requires the owned collection's downloaded depot:
cargo test --manifest-path src-tauri/Cargo.toml live_collection_imports_and_reuses_each_game --lib -- --ignored
```

The separate `import_collection_into_explicit_library` opt-in helper writes to
an explicitly supplied `KITOR_STEAM_IMPORT_LIBRARY` directory. Normal tests never
write to the user's library. The app's Steam dialog is the normal import entry point.

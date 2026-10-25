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

1. Click "Scan Folder" in the Library tab
2. Select a folder containing ScummVM games
3. Kitor will automatically detect compatible games

Or add games manually with the "Add Manually" button.

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
are mapped from `sq1/PATCHES/` into the detected SCI game data directory.

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

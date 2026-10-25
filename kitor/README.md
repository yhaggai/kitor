# Kitor - ScummVM Translation Patch Manager

A desktop application built with Tauri + React for managing ScummVM games and community translation patches.

## Features

- 🎮 **Game Library** - Automatically scan and detect ScummVM games
- 🌍 **Translation Patches** - Browse and install community translation patches
- 📦 **One-Click Install** - Download, backup, and apply patches automatically
- 🔄 **Backup & Restore** - Automatically backup original game files
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
4. Original files are automatically backed up

### Launching Games

Click the "Play" button on any game card to launch it through ScummVM.

## Patch Registry Format

Create a `patches/registry.json` file with patch metadata:

```json
{
  "patches": [
    {
      "game_scummvm_id": "monkey",
      "language": "ja",
      "version": "1.0.0",
      "author": "Community Team",
      "completion": 100,
      "download_url": "https://example.com/patch.zip",
      "installer_url": null,
      "source_type": "github",
      "checksum": "sha256:...",
      "file_size": 5242880,
      "description": "Full Japanese translation"
    }
  ]
}
```

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

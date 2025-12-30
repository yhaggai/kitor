# Kitor Setup Guide

## Step-by-Step Installation

### 1. Install Rust

Kitor uses Tauri which requires Rust to compile the backend.

**Windows:**
```powershell
# Option 1: Using winget
winget install Rustlang.Rustup

# Option 2: Download installer
# Visit https://rustup.rs/ and download rustup-init.exe
```

After installation, restart your terminal and verify:
```bash
cargo --version
rustc --version
```

### 2. Verify Bun Installation

Bun is already installed at `C:\Users\haggai\.bun\bin\bun.exe`

Add to PATH or use the full path. Verify installation:
```bash
bun --version
```

### 3. Install ScummVM

Download and install ScummVM from: https://www.scummvm.org/downloads/

Kitor will automatically detect ScummVM once installed.

### 4. Build and Run Kitor

**Development mode (with hot-reload):**
```bash
cd kitor
bun run tauri dev
```

**Production build:**
```bash
cd kitor
bun run tauri build
```

The executable will be created in `src-tauri/target/release/kitor.exe`

## Troubleshooting

### Rust Not Found

If you get "cargo: command not found":
1. Restart your terminal/IDE after installing Rust
2. Check if `%USERPROFILE%\.cargo\bin` is in your PATH
3. Run: `refreshenv` in PowerShell

### Build Errors

If you encounter build errors:
```bash
# Update Rust
rustup update

# Clean and rebuild
cd kitor/src-tauri
cargo clean
cd ..
bun run tauri build
```

### ScummVM Not Detected

Kitor checks these paths for ScummVM:
- `C:\Program Files\ScummVM\scummvm.exe`
- `C:\Program Files (x86)\ScummVM\scummvm.exe`
- System PATH

If installed elsewhere, Kitor will show "ScummVM not found" in Settings.

### Database Issues

If you encounter database errors, delete the database and restart:
```powershell
# Database location
rm $env:APPDATA\kitor\kitor.db
```

## Development Workflow

### Running Tests

```bash
# Frontend
cd kitor
bun test

# Backend
cd kitor/src-tauri
cargo test
```

### Code Structure

**Adding a new Tauri command:**
1. Create the command in `src-tauri/src/commands/`
2. Add to `invoke_handler` in `src-tauri/src/lib.rs`
3. Call from frontend using `invoke('command_name')`

**Adding a new UI component:**
1. Create component in `src/components/`
2. Import and use in your pages
3. Use shadcn/ui components from `src/components/ui/`

### Database Migrations

The database is auto-created on first run. To modify schema:
1. Update models in `src-tauri/src/models/`
2. Update `create_tables()` in `src-tauri/src/db.rs`
3. Delete old database and restart

## Building for Distribution

### Windows Installer

```bash
cd kitor
bun run tauri build
```

Creates:
- `src-tauri/target/release/kitor.exe` - Portable executable
- `src-tauri/target/release/bundle/nsis/kitor_0.1.0_x64-setup.exe` - Installer

### Optimizations

For smaller binary size, edit `src-tauri/Cargo.toml`:
```toml
[profile.release]
strip = true
lto = true
codegen-units = 1
```

## Next Steps

1. ✅ Install Rust
2. ✅ Verify Bun
3. ✅ Install ScummVM
4. ✅ Run `bun run tauri dev`
5. 🎮 Add some games
6. 🌍 Install translation patches

For more information, see [README.md](README.md)





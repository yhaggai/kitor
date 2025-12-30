# Kitor API Reference

Quick reference for all implemented Tauri commands and their usage.

## Game Commands

### `get_all_games()`
Get all games in the library.

**Returns**: `Promise<Game[]>`

```typescript
const games = await invoke('get_all_games');
```

### `scan_games(scanPath: string)`
Scan a directory for ScummVM games.

**Parameters**:
- `scanPath`: Absolute path to directory

**Returns**: `Promise<Game[]>` - Newly found games

```typescript
const newGames = await invoke('scan_games', { 
  scanPath: 'C:\\Games\\ScummVM' 
});
```

### `add_game_manually(scummvmId: string, title: string, installPath: string)`
Manually add a game to the library.

**Parameters**:
- `scummvmId`: ScummVM game ID
- `title`: Display title
- `installPath`: Game installation path

**Returns**: `Promise<Game>`

```typescript
const game = await invoke('add_game_manually', {
  scummvmId: 'monkey',
  title: 'The Secret of Monkey Island',
  installPath: 'C:\\Games\\monkey-island'
});
```

### `remove_game(id: number)`
Remove a game from the library.

**Parameters**:
- `id`: Game database ID

**Returns**: `Promise<void>`

```typescript
await invoke('remove_game', { id: 1 });
```

### `get_game_by_id(id: number)`
Get a specific game by ID.

**Parameters**:
- `id`: Game database ID

**Returns**: `Promise<Game | null>`

```typescript
const game = await invoke('get_game_by_id', { id: 1 });
```

## ScummVM Commands

### `launch_game(gameId: number)`
Launch a game through ScummVM.

**Parameters**:
- `gameId`: Game database ID

**Returns**: `Promise<void>`

```typescript
await invoke('launch_game', { gameId: 1 });
```

### `get_scummvm_path()`
Get the detected ScummVM executable path.

**Returns**: `Promise<string>`

```typescript
const path = await invoke('get_scummvm_path');
// Returns: "C:\\Program Files\\ScummVM\\scummvm.exe"
```

### `verify_scummvm()`
Check if ScummVM is installed and detected.

**Returns**: `Promise<boolean>`

```typescript
const isInstalled = await invoke('verify_scummvm');
```

## Patch Commands

### `get_patches_for_game(gameId: number)`
Get all patches for a specific game.

**Parameters**:
- `gameId`: Game database ID

**Returns**: `Promise<Patch[]>`

```typescript
const patches = await invoke('get_patches_for_game', { gameId: 1 });
```

### `get_all_patches()`
Get all available patches.

**Returns**: `Promise<Patch[]>`

```typescript
const allPatches = await invoke('get_all_patches');
```

### `add_patch(...)`
Add a new patch to the database.

**Parameters**:
- `gameId`: number
- `language`: string (e.g., "ja", "fr", "de")
- `version`: string (e.g., "1.0.0")
- `downloadUrl`: string
- `installerUrl`: string | null
- `sourceType`: string ("github" | "custom")

**Returns**: `Promise<Patch>`

```typescript
const patch = await invoke('add_patch', {
  gameId: 1,
  language: 'ja',
  version: '1.0.0',
  downloadUrl: 'https://example.com/patch.zip',
  installerUrl: null,
  sourceType: 'github'
});
```

### `download_and_install_patch(patchId: number)`
Download and install a patch (with automatic backup).

**Parameters**:
- `patchId`: Patch database ID

**Returns**: `Promise<void>`

**Emits**: `download-progress` events

```typescript
// Listen for progress
const unlisten = await listen('download-progress', (event) => {
  console.log(`Progress: ${event.payload.percentage}%`);
});

// Install patch
await invoke('download_and_install_patch', { patchId: 1 });

// Clean up listener
unlisten();
```

### `get_installed_patches(gameId: number)`
Get all patches installed for a game.

**Parameters**:
- `gameId`: Game database ID

**Returns**: `Promise<InstalledPatch[]>`

```typescript
const installed = await invoke('get_installed_patches', { gameId: 1 });
```

### `restore_backup(installedPatchId: number)`
Restore game from backup (undo patch).

**Parameters**:
- `installedPatchId`: InstalledPatch database ID

**Returns**: `Promise<void>`

```typescript
await invoke('restore_backup', { installedPatchId: 1 });
```

## Events

### `download-progress`
Emitted during patch downloads.

**Payload**:
```typescript
{
  downloaded: number;  // Bytes downloaded
  total: number;       // Total file size
  percentage: number;  // 0-100
}
```

**Usage**:
```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen('download-progress', (event) => {
  const { downloaded, total, percentage } = event.payload;
  console.log(`Downloaded ${downloaded}/${total} bytes (${percentage}%)`);
});

// Clean up when done
unlisten();
```

## Type Definitions

### Game
```typescript
interface Game {
  id: number;
  scummvm_id: string;
  title: string;
  install_path: string;
  cover_url?: string;
  detected_version?: string;
  added_date: string;  // ISO datetime
  last_played?: string;  // ISO datetime
}
```

### Patch
```typescript
interface Patch {
  id: number;
  game_id: number;
  language: string;
  version: string;
  download_url: string;
  installer_url?: string;
  source_type: string;
  file_size?: number;
  checksum?: string;
  author?: string;
  description?: string;
  completion_percentage?: number;
}
```

### InstalledPatch
```typescript
interface InstalledPatch {
  id: number;
  game_id: number;
  patch_id: number;
  installed_date: string;  // ISO datetime
  status: "active" | "disabled" | "failed";
  backup_path?: string;
}
```

## Error Handling

All commands return `Result<T, String>` on the Rust side, which translates to:
- **Success**: Resolves with the data
- **Error**: Rejects with error message string

```typescript
try {
  const games = await invoke('get_all_games');
  console.log('Games loaded:', games);
} catch (error) {
  console.error('Failed to load games:', error);
  // error is a string from Rust
}
```

## Database Location

- **Windows**: `%APPDATA%\kitor\kitor.db`
- **Linux**: `~/.local/share/kitor/kitor.db`
- **macOS**: `~/Library/Application Support/kitor/kitor.db`

## File Locations

### Downloads
Temporary patch downloads: `<APP_DATA>/downloads/`

### Backups
Game backups: `<APP_DATA>/backups/<scummvm_id>/<timestamp>/`

### Patches
Patch registry: `<PROJECT_ROOT>/patches/registry.json`

## Common Patterns

### Loading Games on Startup
```typescript
useEffect(() => {
  async function loadGames() {
    try {
      const games = await invoke('get_all_games');
      setGames(games);
    } catch (error) {
      console.error('Failed to load games:', error);
    }
  }
  loadGames();
}, []);
```

### Installing Patch with Progress
```typescript
const [progress, setProgress] = useState(0);

useEffect(() => {
  const unlisten = listen('download-progress', (event) => {
    setProgress(event.payload.percentage);
  });
  return () => { unlisten.then(fn => fn()); };
}, []);

const installPatch = async (patchId: number) => {
  try {
    await invoke('download_and_install_patch', { patchId });
    alert('Patch installed successfully!');
  } catch (error) {
    alert(`Failed to install patch: ${error}`);
  }
};
```

### Launching Game
```typescript
const handleLaunch = async (gameId: number) => {
  try {
    await invoke('launch_game', { gameId });
  } catch (error) {
    alert(`Failed to launch game: ${error}`);
  }
};
```

## Tips

1. Always wrap `invoke` calls in try-catch blocks
2. Clean up event listeners when components unmount
3. Check ScummVM installation before launching games
4. Show progress bars during downloads
5. Confirm before removing games or restoring backups






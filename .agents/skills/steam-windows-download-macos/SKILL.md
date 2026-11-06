---
name: steam-windows-download-macos
description: Download an owned Steam game's Windows files on macOS through the installed Steam client, including games whose Steam listing lacks Mac support. Use for cross-platform Steam downloads and their integration into a launcher; downloading alone does not establish runtime compatibility.
---

# Download Steam Windows game files on macOS

Use the user's installed, authenticated Steam client to download the game's Windows depots. The user does not need to type console commands. Prefer this demonstrated method over installing SteamCMD or asking for a Steam Web API key. Steam handles authentication and entitlement; OpenID/Web API ownership information does not itself authorize content downloads.

## Identify the content

- Resolve the exact game edition, Steam app ID, and Windows depot IDs from current metadata. A game can require several shared, language, and Windows-specific depots; do not assume a single depot contains the whole game. Do not substitute the example IDs for another title.
- Confirm Steam is installed and initialized. The user must sign in to an account entitled to the game. If sign-in or Steam Guard is required, let the user complete it in Steam; do not extract session credentials from local files.
- Treat downloading and running as separate tasks. ScummVM supports particular game data, not arbitrary Windows executables. Other games may need Wine/CrossOver or another runtime and may not work at all. Follow the user's requested scope; downloading does not imply installing a runtime, applying a translation, or purchasing a game.

## Invoke the local Steam client

On the Mac where this workflow was verified, the active binary was:

```text
~/Library/Application Support/Steam/Steam.AppBundle/Steam/Contents/MacOS/steam_osx
```

The installed bootstrap binary was `/Applications/Steam.app/Contents/MacOS/steam_osx`. Locate and validate the available binary on the current machine rather than assuming these paths always exist. The active binary above successfully forwarded the command to an already-running Steam client.

Invoke the binary with separate arguments, without shell interpolation:

```text
-console +download_depot APP_ID DEPOT_ID
```

Use a process API such as `subprocess.Popen([str(binary), "-console", "+download_depot", str(app_id), str(depot_id)])` or Rust `Command::args`. Validate IDs as numeric values. Let Steam resolve the current depot manifest unless a specific known game version is required. A manifest can be supplied as the next argument when supported by the current client.

Steam can show its Console tab or a sign-in window. Do not promise a completely invisible download. Starting the command or receiving exit code zero is not evidence of download completion: the process may merely forward the request, or may remain alive when Steam starts cold.

## Verify completion

Before issuing the command, record the length of Steam's console log, typically:

```text
~/Library/Application Support/Steam/logs/console_log.txt
```

Monitor new complete lines after that offset, accounting for truncated/rotated logs and lines split between reads. For sequential multi-depot downloads, track each requested depot separately. Avoid duplicate in-flight requests for the same depot.

Observed messages:

```text
Downloading depot 32461 (40 files, 1378 MB) ...
Depot download complete : "/.../MacOS\steamapps\content\app_32460\depot_32461" (manifest ...)
```

- Match the expected app/depot exactly; old completion entries and other downloads do not prove this request succeeded.
- Steam's printed macOS destination may mix `/` and `\`. Normalize separators for locating the actual directory, validate that it is the expected Steam content destination, and inspect it on disk.
- Check entitlement/download errors and return an actionable explanation. On authentication or ownership failure, stop retrying until the user resolves it. Use a bounded wait; a timeout means completion is unknown, not that Steam stopped downloading.
- Verify expected files and nonempty game data after completion. Report the actual folder and whether only download, or also runtime launch, has been verified.
- Do not present Steam's global download speed or preallocated file sizes as game-specific completion percentages. In a UI, use an indeterminate progress bar unless reliable per-depot progress is available.

The demonstrated destination was beneath the active client binary, **not** the usual `Steam/steamapps/common` library:

```text
~/Library/Application Support/Steam/Steam.AppBundle/Steam/Contents/MacOS/steamapps/content/app_32460/depot_32461
```

Raw depot downloads may have no normal installed-game appmanifest. Do not equate Steam ownership with installation or assume ordinary library discovery will find these files.

## Verified example: Monkey Island 2 Special Edition

Verified locally on 2026-10-25:

- Game: **Monkey Island 2 Special Edition: LeChuck's Revenge**.
- App `32460`, Windows depot `32461`; observed manifest `7019039573050115999`.
- Download approximately 1.35 GiB; expanded content approximately 1.80 GiB.
- Expected content includes `Monkey2.exe`, `monkey2.pak`, `steam_api.dll`, `audio/`, and `installers/`.
- `Monkey2.exe` was identified as a Windows PE32 executable.
- Installed ScummVM **2026.1.0** directly detected the folder as `scumm:monkey2` (`SE/DOS/English`) and displayed the classic game's opening screen. No PAK extraction was needed for that tested runtime. Do not generalize this to older ScummVM builds or other games.

For a requested ScummVM launch, first use its `--detect --path=GAME_FOLDER`, then launch the detected ID, e.g. `--no-fullscreen --path=GAME_FOLDER scumm:monkey2`. Pass each option as a separate process argument and verify the game window. This loads classic game data; it does not run the remastered Windows executable.

## Launcher integration, when requested

For a curated compatible title on macOS, one Steam button can explain that the Windows version will be downloaded. A **Next** action starts the operation and displays progress; leave normal Steam behavior for other titles. Unsupported-by-Steam does not establish compatibility with the launcher's runtime, so gate this flow using explicit, verified per-game metadata.

After all required depots finish, import their known game content into a stable managed directory rather than depending on files inside Steam's application bundle. Stage and verify copied bytes before registering the installation. Do not overwrite an existing installation, saves, patch state, or recovery backups; do not import unrelated patches merely because they share the depot folder. Preserve relative paths and reject escaping paths or unexpected symlinks. If registration fails after import, retain the verified folder and report how to add it manually.

Keep download status, game installation, and patch installation distinct. Support retry without overwriting a valid managed installation. Closing a progress dialog does not cancel Steam's download; only claim cancellation when it is actually supported and verified.

Steam's console command and log format are an unofficial integration surface. If they change, inspect current behavior and adapt, rather than treating the example as a permanent API contract.

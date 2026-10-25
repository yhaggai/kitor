# Windows Hebrew patch — alpha 2

This patch is for the Windows Steam version of UNDERTALE. It includes the complete draft Hebrew catalog and the speech-bubble margin correction.

**The Windows game itself has not yet been run or playtested.** The archive compiles and reloads successfully; installer and rollback tests were performed on disposable Windows game files on macOS.

## Install

1. Install Python 3.10+ with the Windows `py` launcher if it is not already installed.
2. Save your progress and close UNDERTALE.
3. Extract the entire Windows patch ZIP into one folder.
4. Double-click **Install Hebrew Windows.cmd**.
5. Start UNDERTALE through Steam, with the in-game language set to **English**.

The installer finds Steam through the Windows registry and Steam library folders. For a custom location, open a command prompt in the extracted patch folder:

```bat
py -3 install.py --platform windows --game-dir "D:\SteamLibrary\steamapps\common\Undertale"
```

Only `data.win` is replaced. The executable, Steam integration and saves are preserved. Checksums protect against applying this patch to another version or mod.

## Roll back

The installer prints the backup folder under `%LOCALAPPDATA%\Undertale Hebrew\backups`. Close the game, then drag that backup folder onto **Restore Original Windows.cmd**. Alternatively:

```bat
py -3 install.py --rollback "C:\path\to\your\backup-folder"
```

Keep the backup folder intact. Rollback verifies the original backup and refuses to overwrite a different mod or game update.

## Compatibility and limits

- Steam application 391540, Windows depot 391541, manifest 3413030246586915448.
- Expected original `data.win` SHA-256: `b718f8223a5bb31979ffeed10be6140c857b882fc0d0462b89d6287ae38c81c7`.
- 16,455 catalog entries, including 15,227 Hebrew drafts and 1,228 unchanged entries.
- All 20 fonts extended; 378 ordinary drawing calls redirected; Steam integration retained.
- Graphical text remains English. The translation needs proofreading and gameplay review.
- This is a binary patch requiring your own original game. No complete game archive or executable is included.

# Police Quest: In Pursuit of the Death Angel — Hebrew

Status: packaged, awaiting native Windows/macOS and owned-game validation.

Install and uninstall through Kitor’s declarative lifecycle. Only the files listed in patch.json are affected; originals are backed up individually outside the game. Uninstall restores affected originals and removes patch-created files, preserving unrelated files and saves. Never run the upstream installer against the game.

See provenance.json for exact source URLs, edition restrictions, dependencies and validation gaps. The same payload supports Windows and macOS runtime plans; this is not a claim of tested native builds.

## Steam on macOS

Kitor downloads Police Quest Collection app 494740, depot 494741 through the
signed-in Steam client. It imports only the reviewed AGI/EGA data from `PQ1/EGA`,
using SHA-256 fingerprints in `kitor/src-tauri/data/steam_products.json`.
The VGA remake in `PQ1` is a different game and is excluded.

Verified on macOS on 2026-10-29: managed import and retry preserve saves;
ScummVM 2026.4.0 detects `agi:pq1` (2.0G 1987-12-03/DOS/English) and starts
AGI v2.917. This does not establish complete gameplay compatibility.

The published Hebrew package selects the first upstream VOL.0/LOGDIR variants.
Steam matches the second variants (MD5 `4d44bf0e438ba7c0b05c17be4dea36c5`
and `d194e5d88363095f55d5096b8e32fbbb`), so the existing package will safely
reject those inputs. Hebrew installation for this edition still needs a
separate validated package; do not bypass its edition guards.

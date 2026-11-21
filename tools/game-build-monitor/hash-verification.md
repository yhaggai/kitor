# Supported builds recovered from original files

On 2026-11-16, the original SHA-256 values from the patch plans below matched the installed files or retained original backups. For the cached Steam depots, each verified original also matched the depot file's path, size, and SHA-1. Where a file was already patched, the SHA-256 and depot checks used the retained original. These checks establish the patch's source-file baseline.

| Patch | Store and platform | Build evidence | SHA-256 matches |
| --- | --- | --- | --- |
| [UNDERTALE](../../patches-workspace/undertale/he/patch.json) | Steam macOS | App 391540, cached depot 391542 manifest `1593483711954100372`. Verified the backed-up `game.ios`. | 1 of 1 |
| [Space Quest I (EGA)](../../patches-workspace/sq1/he/patch.json) | Steam Windows and macOS | App 10110, cached depot 10111 manifest `6725209980292758062`. Verified the `2016_SpaceQuestCollection/sq1` data against both platform plans, which expect the same originals. | 9 of 9 per plan |
| [Day of the Tentacle Remastered](../../patches-workspace/day-of-the-tentacle-remastered/he/patch.osx.candidate.json) | Steam macOS | App 388210, cached depot 388214 manifest `2376453229416774441`. Verified the original archive and the other guarded files from the installation and backups. | 5 of 5 |
| [Jenny LeClue](../../patches-workspace/jenny-leclue/he/patch.json) | Steam macOS | Steam's content log records build 24043907 and depot 319872 manifest `8849404721773343611` committed to the same Steam install the patch journal references. Verified the originals from that journal. | 4 of 4 |
| [Disco Elysium](../../patches-workspace/disco-elysium/he/patch.json) | Steam macOS | App 632470, cached depot 632473 manifest `3514503533443935223`. Verified the five guarded files in the local Steam installation. | 5 of 5 |
| [The Secret of Monkey Island Special Edition native patch](../../patches-workspace/monkey-se-native/he/patch.windows.candidate.json) | GOG Windows | The installation's `goggame-1207666253.info` records product 1207666253 and build `50953075782223848`. Verified all four guarded files in that installation. | 4 of 4 |

## Owlboy Steam build 26574659

On 2026-11-17, the clean Windows and macOS depot files were checked against their Steam SHA-1 hashes before comparison against the patch's SHA-256 requirements.

- macOS depot 115805, manifest `409618040785163464`. All 17 original hashes matched the bundled `steam-138-4809/patch.zip` that Kitor selects on macOS.
- Windows depot 115801, manifest `4002365934342145975`. Seven of the 17 original hashes differed from release `0.2.0-beta.2-kitor2-download1`. The registry records this mismatch for that exact manifest and patch version. The supported Windows manifest remains unrecorded.

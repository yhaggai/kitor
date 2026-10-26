# One-step classic Hebrew patch

Catalog 176, game ID `tentacle`, upstream `DOTT-WIP2`.

The native installer accepts supported English CD classic files directly and
patches only `tentacle.000` and `tentacle.001`, backing up those two originals.
Existing speech files and saves are untouched. Remastered ownership is optional.

Alternatively it reads an owned Day of the Tentacle Remastered archive and creates `hebrew/tentacle.000`, `hebrew/tentacle.001`, and
`hebrew/monster.so3` beside the game/app. Kitor launches these through ScummVM.
It does not translate the remastered presentation or alter its signed Mac bundle.
The remastered Hebrew prototype is a separate patch.

No Python, remonster, xdelta, or FFmpeg installation is needed by the end user.
The Rust reconstruction decoder is built into Kitor and the standalone
`tentacle_patch` binary. It verifies the source archive SHA-256, bounds every
copy range, verifies outputs, and uses Kitor's existing journaled lifecycle.
For extraction, existing destination files are refused. Direct-classic uninstall
restores its two originals byte-for-byte; extraction uninstall removes recorded files;
changed files cause a conflict, and saves/unrelated files remain untouched.

## Local build

```sh
cargo build --release --manifest-path kitor/tools/lifecycle-harness/Cargo.toml --bin tentacle_patch
python3 kitor/tools/build_patch_packages.py --output dist/patches patches-workspace/tentacle/he/patch.json
```

Run the resulting native executable with:

```text
tentacle_patch install GAME_FOLDER RECOVERY_FOLDER
tentacle_patch uninstall GAME_FOLDER JOURNAL_FOLDER
```

Use an absolute recovery directory outside the game folder. The install command
prints its journal path. Kitor manages this automatically through its Install
and Restore buttons. Windows and macOS use the same Rust implementation, with
platform-specific archive paths declared in `patch.json`.

## Reproduction and provenance

`build_recipes.py` takes an owned original `tenta.cle`, applies the inspected
upstream deltas using xdelta3 at build time, and generates bounded source-copy
recipes. Sound indexes/timing tags come from the pinned remonstered source;
all sound bytes are read from the user's archive at installation time.
`recipe-build.json` pins input/output hashes. See `provenance.json` for source
URLs, source revisions, credits, restrictions, and actual validation status.

The upstream announcement permits personal use and prohibits redistribution.
These are local test artifacts. `release_ready` remains false; do not publish
translation payloads or compiled binaries containing them without permission.

## Edition separation

Classic has ID `tentacle` and runs through ScummVM. The separate
`day-of-the-tentacle-remastered` candidate runs natively and has its own version,
manifest and recovery journal. Scanning a Remastered installation offers both
choices. The two patches own disjoint paths and can be uninstalled independently.
Unknown classic language/floppy editions fail checksum validation before mutation;
only the verified English CD resource pair is currently supported.

Windows extraction is pinned to Steam depot 388211 manifest 8150331470367542821;
Mac extraction is pinned to depot 388214 manifest 2376453229416774441. Both were
obtained from the owned Steam installation and produce identical output hashes.
Native Windows runtime testing is still required; cross-compilation is not that test.

# Owned-game patch workspace

Ownership snapshot: 25 October 2026. All 28 catalog matches are represented here.

Installable packages are indexed in the Kitor registry. New packages are published as unverified prereleases; game-specific Windows/macOS gameplay validation remains outstanding. Some entries need manual selection/detection support, and modern games must be launched through Steam/GOG.

## Contract

Every releasable patch needs Windows and macOS artifacts, a declared file plan, source hashes, edition/dependency information, and selective install/uninstall through Kitor. Back up only affected existing files; restore those originals and remove patch-created files. Never copy or clear the entire game directory. See [specification.json](specification.json).

Portable data packages use Kitor and the appropriate native game/ScummVM runtime; packaging a ZIP does not compile a new native installer. Dependencies and exceptions live in each provenance.json.

## Inventory

| # | Game | Owned on | Status |
|---|---|---|---|
| 1 | [Simon The sorcerer 25th Ann. edition](simon1-25th/he/provenance.json) | gog | Hebrew built in; no external patch |
| 4 | [The Secret of Monkey Island](monkey-se/he/provenance.json) | gog | Windows + macOS packages; gameplay unverified |
| 5 | [Monkey Island 2: LeChuck's Revenge](monkey2-se/he/provenance.json) | steam | Windows + macOS packages; gameplay unverified |
| 16 | [Police Quest: In Pursuit of the Death Angel](pq1/he/provenance.json) | steam | Windows + macOS packages; gameplay unverified |
| 17 | [Space Quest: The Sarien Encounter (VGA)](sq1sci/he/provenance.json) | steam | Windows + macOS packages; gameplay unverified |
| 18 | [Space Quest III: The Pirates of Pestulon](sq3/he/provenance.json) | steam, gog | Windows + macOS packages; gameplay unverified |
| 19 | [Torin's Passage](torin/he/provenance.json) | gog | Windows + macOS packages; gameplay unverified |
| 22 | [The Stanley Parable (2013)](stanley-parable/he/provenance.json) | steam | Windows candidate; macOS pending |
| 27 | [The Legend of Kyrandia (Book One)](kyra1-cd/he/provenance.json) | gog | Windows + macOS packages; gameplay unverified |
| 36 | [Tesla Effect: A Tex Murphy Adventure](tesla-effect/he/provenance.json) | steam | Windows candidate; macOS pending |
| 39 | [She Sees Red - Interactive Movie](she-sees-red/he/provenance.json) | steam | Hebrew built in; no external patch |
| 68 | [Spy Fox In Dry Cereal](spyfox/he/provenance.json) | steam | Windows + macOS packages; gameplay unverified |
| 69 | [Pajama Sam: No Need to Hide When It's Dark Outside](pajama/he/provenance.json) | gog | Windows + macOS packages; gameplay unverified |
| 72 | [Space Quest V: The Next Mutation](sq5/he/provenance.json) | steam | Windows + macOS packages; gameplay unverified |
| 75 | [The Vanishing of Ethan Carter](ethan-carter/he/provenance.json) | steam | Windows candidate; macOS pending |
| 76 | [Toonstruck](toonstruck/he/provenance.json) | gog | Windows + macOS packages; gameplay unverified |
| 77 | [Life is Strange Episode 1-3](life-is-strange/he/provenance.json) | steam | Windows candidate; macOS pending |
| 87 | [Portal 2](portal2/he/provenance.json) | steam | Experimental Windows candidate; macOS pending |
| 90 | [Gobliins 2: The Prince Buffoon](gob2/he/provenance.json) | gog | Windows + macOS packages; gameplay unverified |
| 97 | [The Shivah](shivah/he/provenance.json) | steam | Owned edition incompatible with catalog patch |
| 99 | [The Journey Down](journey-down1/he/provenance.json) | steam | Windows candidate; macOS pending |
| 107 | [The Longest Journey](tlj/he/provenance.json) | steam, gog | Windows + macOS packages; gameplay unverified |
| 112 | [Beneath A Steel Sky](sky/he/provenance.json) | gog | Windows + macOS packages; gameplay unverified |
| 121 | [Kathy Rain: Director's Cut](kathy-rain-dc/he/provenance.json) | steam | Windows candidate; macOS pending |
| 130 | [Space Quest II: Vohaul's Revenge](sq2/he/provenance.json) | steam, gog | Windows + macOS packages; gameplay unverified |
| 136 | [Space Quest 1: The Sarien Encounter (EGA)](sq1/he/provenance.json) | gog | Windows + macOS packages; gameplay unverified |
| 160 | [The Drifter](drifter/he/provenance.json) | steam | Bundled Windows + macOS; gameplay unverified |
| 164 | [Gemini Rue](gemini-rue/he/provenance.json) | steam | Owned edition incompatible with catalog patch |

## Build and verification

Run from the repository root:

```sh
python3 kitor/tools/build_patch_packages.py patches-workspace/<game>/he/patch.json --output dist/patches
python3 kitor/tools/build_patch_packages.py patches-workspace/<game>/he/patch.windows.candidate.json --platform windows --output dist/patches
python3 -m unittest discover -s kitor/tools -p "test_*.py"
python3 kitor/tools/run_lifecycle_tests.py --nocapture
python3 kitor/tools/update_owned_patch_catalog.py
```

The [machine-readable inventory](owned-catalog.json) records local artifact paths and SHA-256 hashes. The [validation record](validation.json) distinguishes fixture results from native gameplay checks.

Journey Down and Portal 2 have payloads exceeding GitHub single-file limits. Those assets are excluded from Git and hydrated from private, SHA256-pinned build-input release assets. CI builds both dual-platform packages and Windows candidates.

Release automation requires explicit release_ready=true in provenance.json and validated dual-platform manifests. All imported entries remain false. Raw upstream installers and ownership metadata are excluded from release ZIPs.

## Original Owlboy translation

[Owlboy Hebrew](owlboy/he/README.md) is imported from the separately published
`yhaggai/owlboy-hebrew` source repository. Its full 4,537-entry translation and
corrected flag are packaged as a macOS candidate at
`owlboy/he/patch.osx.candidate.json`. Windows edition validation remains pending.

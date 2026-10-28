# Patch package validation

`build_patch_packages.py` builds deterministic data archives from the exact
`Package`, `Plan`, `Change`, and `Source` schema consumed by
`src-tauri/src/services/patch_lifecycle.rs`. It does not compile executables,
execute publisher installers, install games, or certify game compatibility.

From the repository root:

```sh
python3 -m unittest discover -s kitor/tools -p test_build_patch_packages.py
python3 kitor/tools/build_patch_packages.py --check patches-workspace/undertale/he/patch.json
python3 kitor/tools/build_patch_packages.py --output dist/patches patches-workspace/undertale/he/patch.json
python3 kitor/tools/build_patch_packages.py --universal --output dist/patches patches-workspace/undertale/he/patch.json
```

The default builds separate Windows and macOS ZIPs. `--universal` builds the
single filename expected by the current registry and validates all declared
platforms. Both required platforms must validate before artifacts are written.
Only the manifest and its referenced assets are included. Undeclared upstream
scripts, archives, and development files are excluded. Symlink assets, unsafe
portable paths, case collisions, missing files, and SHA256 mismatches fail.
Blocked or built-in-only entries must not have installable `patch.json` files;
pass only concrete installable manifests, never a provenance descriptor.

Installation uses Kitor's journaled lifecycle; uninstallation uses its recorded
selective restoration. Data-only copy patches need no compiler. BSDIFF plans
require Python 3 on the target; Xdelta plans require the declared hashed Windows
tool or `xdelta3` on macOS. Bundling a verified tool does not prove its native
compatibility, architecture, provenance, or functionality. Those facts belong in
each patch's provenance and require separate testing.

`validate-patch-packages.yml` schedules packaging checks and the existing native
Rust lifecycle fixture tests on Windows and macOS. Merely adding this workflow
does not mean those checks passed. Fixture success also does not replace testing
the patch against the actual supported edition of a game. No release is
published by this validation workflow.

The lightweight `lifecycle-harness` compiles the actual Rust lifecycle and registry
modules without Tauri. Run from the repository root; the runner selects the working directory
required by the existing module fixtures:

```sh
python3 kitor/tools/run_lifecycle_tests.py
```

Its additional test exercises real copy-only payload manifests on disposable
layouts: one original sentinel, all declared copied bytes, backup cardinality,
selective restoration/removal, and unrelated saves before/after installation.
Delta packages needing original game bytes are excluded from that additional
test, and the real Undertale archive test remains explicitly ignored by default.
Both platform plans on one host are not native validation of the other OS.

For explicitly staged single-platform candidates, use `--platform windows` or
`--platform osx` with that candidate manifest path. This builds only the declared
platform and never creates a missing-platform artifact; it cannot be combined
with `--universal`. Default builds still require both Windows and macOS.
A provenance sidecar marked `blocked`, `no_patch_required`, or `built_in` prevents
packaging; `packaged_unverified` is allowed for local work. A successful build
is not a publication approval.

Optional `licenses/` UTF-8 `.txt`, `.md`, or extensionless text files travel in
every archive without changing the install file plan. Symlinks and binary files
are rejected. Original and installed MD5 guards are supported for legacy
publishers; payload assets remain SHA256-verified.

The local fixture runner and CI include Windows candidates by default. CI first
hydrates the two large Git-ignored payloads from the private GitHub build-input
release `patch-build-inputs-20261025`; `patch-build-inputs.json` pins their exact
sizes and SHA256 hashes. `hydrate_patch_inputs.py` uses authenticated `gh release
download`, verifies every byte, and never executes upstream installers. Missing
or altered inputs fail the job. Upload the pinned source assets before running
CI. Existing local inputs are verified without downloading.

Pushes to `v1`/`main`, pull requests, and manual dispatch build artifact-only
Windows/macOS packages plus Windows candidate archives. Tests and package builds
are required on every run. Uploading the ZIPs is optional: enable `upload_artifacts`
on a manual dispatch to retain `unverified-patch-builds-*` for one day. Requested
upload failures still fail that manual run; ordinary validation does not consume
artifact storage. Building packages never changes `release_ready` or publishes a release. Native CI fixtures do not certify gameplay or edition
compatibility. Fork PRs without access to the private input release cannot hydrate
those files and will fail explicitly.

To focus a local payload check, set `KITOR_PAYLOAD_GAME` to one game directory
name or comma-separated names. Leave it unset for the full copy payload suite.
`KITOR_SKIP_CANDIDATES=1` remains available for an explicit restricted local run.

`reconstruct` sources use the native bounded source-range decoder. Each recipe
pins the input SHA-256 (and optionally explicitly verified compatible inputs),
declares its exact output length, and contains only copy ranges and literal bytes.
The manifest also pins the recipe and final output SHA-256. No external runtime
is required. Tentacle direct-classic and optional Remastered extraction plans are
separate manifests; use `patch.remastered-source.json` for the latter.

---
name: kitor-patches
description: Build, update, or review Kitor game patches with Windows and macOS artifacts, installation, and selective backup and uninstall. Use for Kitor patch packaging and lifecycle work.
---

# Kitor patches

Every Kitor patch must provide usable Windows and macOS builds, an install method, and an uninstall method. Apply this contract to each patch, including changes performed by bundled tools and post-install scripts.

## Platform deliverables

- Produce patch artifacts for both Windows and macOS. Compile platform-specific executables and dependencies for their intended targets; a Windows executable alone does not satisfy macOS support.
- Shared platform-independent patch data may be reused, but both platforms must have working installation and uninstallation entry points integrated with Kitor.
- Identify supported game editions/versions, patch version, target architectures, required dependencies, and platform artifacts. Validate compatibility before modifying the game. Do not claim an unbuilt or untested target is verified; report missing builds or validation explicitly.

## Install and backup contract

- Determine the complete set of files the patch will create, modify, replace, delete, or rename before making changes. Include script-generated changes, not just archive entries. Stage or inspect tools whose effects are otherwise unknown.
- Back up every existing file that will be changed before its first mutation. Preserve its original bytes, relative path, and any metadata needed for correct restoration, including executable permissions where relevant.
- Back up only affected existing files. Never copy the entire game directory as a backup. An individual game archive may need a full-file backup when the patch changes that archive.
- Persist a per-install manifest identifying the game installation, patch/version, relative affected paths, whether each path originally existed, backup locations, and operation status. Track newly created files separately; they have no original to back up.
- Keep backups isolated per installation and preserve the originals across retries, reinstallations, and upgrades. Never replace an original backup with an already patched file. Detect overlapping patches before overwriting another patch's restoration state.
- Ensure backups and the manifest are recoverable before applying changes. On failure, roll back only operations performed by that installation and retain recovery information if rollback fails.
- Resolve all target paths within the intended installation; reject escaping archive or manifest paths. Store backups outside the files the patch can overwrite.

## Uninstall and restore contract

- Restore only existing files changed by this patch, using their recorded backups. Restore deleted originals and reverse renames using the same recorded file operations.
- Remove only files created by this patch. Remove patch-created directories only when empty.
- Leave untouched game files, saves, settings, and unrelated files in place. Never clear or replace the game directory during restoration.
- If an affected file changed again after installation or is owned by another active patch, detect the conflict and avoid silently destroying later changes. Record installed hashes or equivalent evidence for this check.
- Report missing or invalid backups rather than claiming a successful uninstall. Retain the manifest and needed backups until restoration succeeds; retries must not damage restored originals or unrelated files.

## Kitor integration and verification

Inspect the current repository before implementing. Useful starting points are `patches-workspace/<game>/<language>/`, patch metadata and platform scripts, `kitor/src-tauri/src/services/patch_service.rs`, and `kitor/src-tauri/src/services/file_service.rs`. Existing code is context, not authority to weaken this contract. A recursive whole-game backup or directory-clearing restore is noncompliant.

Validate install/uninstall on both Windows and macOS with a representative supported game layout. Check that modified originals restore byte-for-byte, patch-created files disappear, and untouched files (including a file added after installation) survive unchanged. Verify backup contents contain only affected originals. Exercise interrupted installation, retry/reinstall, and missing-backup handling when changing lifecycle code. State separately which builds and native platform checks actually ran.

When asked only to define or review the specification, report compliance gaps without expanding the task into changes to runtime behavior.

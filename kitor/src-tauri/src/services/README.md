# Patch backups and recovery

Kitor keeps two kinds of backups because removing a translation and recovering
from a failed update have different destinations.

| Backup | Purpose | Lifetime |
| --- | --- | --- |
| Original game files | Remove the translation and restore the game to its original state, usually English | Kept across patch updates |
| Previous patched files | Recover the translation that was installed before an update | Temporary; removed after a successful update or successful rollback |

Only affected files are backed up. Kitor does not copy the entire game folder.
Files created by a patch have no original backup; the installation journal records
that they should be removed during restoration.

## State machine

These are conceptual lifecycle states, not literal journal status values.
“Installed” means whichever translation version is currently committed.

```mermaid
stateDiagram-v2
    [*] --> Original
    state "Original game" as Original
    state "Installing first translation" as Installing
    state "Translation installed" as Installed
    state "Preparing update" as Preparing
    state "Applying and verifying update" as Applying
    state "New version committed; cleanup pending" as Committed
    state "Restoring previous translation" as Rollback
    state "Rollback blocked; recovery data retained" as RollbackBlocked
    state "Restoring original game" as Restoring
    state "Restore blocked; recovery data retained" as RestoreBlocked

    Original --> Installing: Install; save affected originals
    Installing --> Installed: Installation succeeds
    Installing --> Restoring: Installation fails or is interrupted

    Installed --> Preparing: Request update
    Preparing --> Installed: Validation or staging fails; game unchanged
    Preparing --> Applying: Persist rollback journal and required backups
    Applying --> Rollback: Failure or interruption before commit
    Applying --> Committed: Verify files; atomically commit active journal
    Committed --> Committed: Interruption; keep new version and resume cleanup
    Committed --> Installed: Temporary rollback data removed

    Rollback --> Installed: Previous version restored; temporary data removed
    Rollback --> RollbackBlocked: Recovery cannot complete
    RollbackBlocked --> Rollback: Resolve issue and retry recovery

    Installed --> Restoring: Uninstall translation
    Restoring --> Original: Originals restored; patch-created files removed
    Restoring --> RestoreBlocked: Restoration cannot complete
    RestoreBlocked --> Restoring: Resolve issue and retry restoration

    note right of Preparing
        Reuse permanent original backups.
        Back up newly affected originals only.
        Temporary rollback saves current patched files.
    end note
    note right of Committed
        Commit is the recovery decision point.
        Before commit: restore the previous translation.
        After commit: keep the new translation.
    end note
```

An interruption during preparation leaves the live game unchanged. If a pending
update journal has already been published, recovery processes it before another
operation proceeds. Cleanup is best-effort; leftover temporary artifacts do not
change which version is committed. See the recovery section below for details.

## First installation

Before changing an existing file, Kitor saves its original bytes outside the game
directory. It records which files existed, which files the patch creates, their
hashes, and the progress of the operation in an installation journal.

If installation fails, recovery restores the original files that were changed
and removes files created by that installation. Unrelated files remain untouched.

## Updating an installed translation

For example, when updating translation version 1 to version 2:

1. Verify the installed files and their original backups. Unexpected changes or
   missing backups stop the update before it modifies the game.
2. Prepare and validate version 2 in a temporary working area, using original
   game data. The current implementation uses the installer against this sparse
   working copy; these staging copies are not additional permanent backups.
3. Prepare a temporary rollback journal covering files affected by either
   version. Save their current contents so recovery can restore version 1.
4. Reuse existing original backup references. Save an original backup for files
   newly affected by the update. Reordering the patch's file list does not change
   existing backup references or rewrite those backups.
5. Apply the prepared changes and verify the resulting files. Files no longer
   included in the new patch are restored to their original state or removed if
   the previous patch created them.
6. Atomically replace the active installation journal to commit version 2.
7. Remove the temporary rollback data. The original game backups remain available
   for uninstalling the translation.

There is no permanent history of previous translation versions. After version 2
is committed and cleanup succeeds, the temporary version 1 backup is gone.
Original backups already retained by an installation are not rewritten or
automatically pruned just because a later patch stops using a file.

## Failure and interruption

The active installation journal is the commit point:

- **Before commit:** recovery restores the previous translation from the
  temporary rollback backup. Original backups introduced only by the failed
  update are discarded after successful recovery.
- **After commit:** recovery keeps the new translation and finishes cleanup.

Recovery runs when the installation is next accessed through the lifecycle
operations; a crash does not require guessing which version was installed.
If recovery cannot complete, its journal and required backups are retained for
another attempt. Conflicting later edits are reported rather than silently
overwritten. Cleanup is best-effort, so an interruption or filesystem error can
leave temporary artifacts; they are not intentional version history.

## Uninstall

Uninstall restores original game files using the persistent backups and removes
files created by the translation. Patch-created directories are removed only
when empty. Saves, settings, and other files outside the affected file set are
left alone. Missing backups or unexpected changes to affected files block safe
restoration and must be resolved first.

## Code map

- [patch_lifecycle.rs](patch_lifecycle.rs): initial installation, journal and
  backup references, file verification, and selective restoration.
- [patch_upgrade.rs](patch_upgrade.rs): update staging, temporary rollback,
  commit, recovery, and update regression tests.
- [patch_service.rs](patch_service.rs): application integration.

Older journals with numeric backup filenames remain readable. Installations
created by the earlier updater can continue using their existing
`originals-update-*` directory; the new updater does not create a fresh duplicate
directory for every update or automatically delete older backup directories.

"""Install or remove this preview in a Steam Windows game folder."""
import argparse
import hashlib
import json
import shutil
import tempfile
from pathlib import Path

from apply_bsdiff import apply_patch

PACKAGE = Path(__file__).resolve().parent
MANIFEST = "patch.json"
STATE = ".kitor-moomin-he-preview"


def game_path(root, name):
    parts = name.split("/")
    if any(not p or p in {".", ".."} or "\\" in p or ":" in p for p in parts):
        raise ValueError(f"Invalid patch path: {name}")
    candidate = root
    for part in parts:
        candidate = candidate / part
        if candidate.is_symlink() or not candidate.resolve().is_relative_to(root):
            raise ValueError(f"Path escapes the target folder: {name}")
    return candidate


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def atomic_copy(source, destination):
    with tempfile.NamedTemporaryFile(dir=destination.parent, delete=False) as file:
        temp = Path(file.name)
    try:
        shutil.copy2(source, temp)
        temp.replace(destination)
    finally:
        temp.unlink(missing_ok=True)


def install(root, state):
    manifest = json.loads((PACKAGE / MANIFEST).read_bytes())
    files = manifest["platforms"]["windows"]["files"]
    for entry in files:
        original = game_path(root, entry["path"])
        if digest(original) != entry["original_sha256"]:
            raise ValueError(f"Expected pristine Steam files. Remove the previous patch in Kitor first: {entry['path']}")

    with tempfile.TemporaryDirectory(dir=root, prefix=".moomin-stage-") as directory:
        stage = Path(directory)
        originals = stage / "originals"
        originals.mkdir()
        for index, entry in enumerate(files):
            original = game_path(root, entry["path"])
            backup = originals / str(index)
            shutil.copy2(original, backup)
            source = entry["source"]
            delta = game_path(PACKAGE, source["path"])
            if digest(backup) != entry["original_sha256"] or digest(delta) != source["sha256"]:
                raise ValueError(f"Input changed or archive is damaged: {entry['path']}")
            target = stage / str(index)
            apply_patch(backup, delta, target)
            if digest(target) != entry["installed_sha256"]:
                raise ValueError(f"Output verification failed: {entry['path']}")
            shutil.copymode(backup, target)

        (originals / MANIFEST).write_text(json.dumps(manifest), encoding="utf-8")
        if state.exists():
            raise ValueError("Preview backup already exists. Run uninstall to restore it first.")
        originals.rename(state)
        changed = []
        try:
            for index, entry in enumerate(files):
                target = game_path(root, entry["path"])
                if digest(target) != entry["original_sha256"]:
                    raise ValueError(f"Game file changed during installation: {entry['path']}")
                (stage / str(index)).replace(target)
                changed.append((index, target))
        except BaseException:
            for index, target in reversed(changed):
                atomic_copy(state / str(index), target)
            raise
    print(f"Installed Hebrew preview {manifest['version']}.")


def uninstall(root, state):
    manifest = json.loads((state / MANIFEST).read_bytes())
    files = manifest["platforms"]["windows"]["files"]
    restore = []
    for index, entry in enumerate(files):
        target = game_path(root, entry["path"])
        original = game_path(state, str(index))
        if digest(original) != entry["original_sha256"]:
            raise ValueError(f"Backup does not match the original: {entry['path']}")
        current = digest(target)
        if current == entry["original_sha256"]:
            continue
        if current != entry["installed_sha256"]:
            raise ValueError(f"Cannot restore a file that changed after installation: {entry['path']}")
        restore.append((original, target))

    for original, target in restore:
        atomic_copy(original, target)
        if digest(original) != digest(target):
            raise ValueError(f"Restore verification failed: {target}")
    # Keep any unrelated files that may have been added to the backup directory.
    for index in range(len(files)):
        (state / str(index)).unlink()
    (state / MANIFEST).unlink()
    try:
        state.rmdir()
    except OSError:
        pass
    print("Restored the original game files.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("install", "uninstall"))
    parser.add_argument("game_folder", type=Path, help="Folder containing the Summer directory")
    args = parser.parse_args()
    root = args.game_folder.resolve(strict=True)
    try:
        if not game_path(root, "Summer/Adventure_UE5.exe").is_file():
            raise ValueError("Choose the Steam game folder containing Summer/Adventure_UE5.exe.")
        state = game_path(root, STATE)
        if args.action == "install":
            install(root, state)
        else:
            uninstall(root, state)
    except (OSError, ValueError) as error:
        parser.exit(1, f"{error}\n")


if __name__ == "__main__":
    main()

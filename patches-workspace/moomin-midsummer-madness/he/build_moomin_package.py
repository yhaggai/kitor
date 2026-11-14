"""Build the Moomin Windows candidate from pristine and patched install trees."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath

import bsdiff4


HERE = Path(__file__).resolve().parent
FILES = [
    "Summer/Adventure_UE5_Data/StreamingAssets/aa/StandaloneWindows64/database_assets_all_c77c9f4b2caf03eae7708be1f1ce7d43.bundle",
    "Summer/Adventure_UE5_Data/StreamingAssets/aa/StandaloneWindows64/fonts_assets_all_2ab40165824cec60e96eea3c37ae678d.bundle",
    "Summer/Adventure_UE5_Data/sharedassets0.assets",
    "Summer/Adventure_UE5_Data/StreamingAssets/aa/catalog.bin",
    "Summer/Adventure_UE5_Data/Managed/Assembly-CSharp.dll",
]


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def rooted(root: Path, relative: str) -> Path:
    return root.joinpath(*PurePosixPath(relative).parts)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--original-root", type=Path, required=True,
                        help="Pristine game tree with paths matching the manifest")
    parser.add_argument("--patched-root", type=Path, required=True,
                        help="Final patched game tree with paths matching the manifest")
    parser.add_argument("--database-output", type=Path,
                        help="Use the freshly built database bundle as the patched database input")
    args = parser.parse_args()

    deltas = HERE / "deltas"
    deltas.mkdir(exist_ok=True)
    files = []
    for index, relative in enumerate(FILES):
        source = rooted(args.original_root, relative).read_bytes()
        target_path = args.database_output if index == 0 and args.database_output else rooted(args.patched_root, relative)
        target = target_path.read_bytes()
        delta = bsdiff4.diff(source, target)
        if bsdiff4.patch(source, delta) != target:
            raise ValueError(f"BSDIFF round-trip failed: {relative}")
        delta_name = f"{index:02d}-{PurePosixPath(relative).name}.bsdiff"
        (deltas / delta_name).write_bytes(delta)
        files.append({
            "path": relative,
            "operation": "modify",
            "original_sha256": digest(source),
            "installed_sha256": digest(target),
            "source": {
                "kind": "bsdiff",
                "path": f"deltas/{delta_name}",
                "sha256": digest(delta),
                "input": relative,
            },
        })

    manifest = {
        "schema_version": 1,
        "scummvm_id": "moomin-midsummer-madness",
        "language": "he",
        "version": "0.1.0",
        "release_revision": 1,
        "filename": "moomin-midsummer-madness-he-0.1.0-windows.zip",
        "platforms": {"windows": {"files": files}},
    }
    (HERE / "patch.windows.candidate.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(f"Wrote {len(files)} verified bsdiff files and patch.windows.candidate.json")


if __name__ == "__main__":
    main()

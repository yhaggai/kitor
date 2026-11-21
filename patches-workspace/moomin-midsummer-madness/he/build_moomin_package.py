"""Build the Moomin Windows candidate from pristine and patched install trees."""
from __future__ import annotations

import argparse
import hashlib
import json
import zlib
from pathlib import Path, PurePosixPath



HERE = Path(__file__).resolve().parent
FILES = [
    "Summer/Adventure_UE5_Data/StreamingAssets/aa/StandaloneWindows64/database_assets_all_451c9c2dc5087ceed02f737272b90df7.bundle",
    "Summer/Adventure_UE5_Data/StreamingAssets/aa/StandaloneWindows64/fonts_assets_all_2ab40165824cec60e96eea3c37ae678d.bundle",
    "Summer/Adventure_UE5_Data/sharedassets0.assets",
    "Summer/Adventure_UE5_Data/StreamingAssets/aa/catalog.bin",
    "Summer/Adventure_UE5_Data/Managed/Assembly-CSharp.dll",
]


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def rooted(root: Path, relative: str) -> Path:
    return root.joinpath(*PurePosixPath(relative).parts)


def bundle_crc(blob: bytes) -> int:
    import UnityPy
    bundle = UnityPy.load(blob).file
    # SerializedFile.save() can alter pristine bytes. CRC covers raw members.
    return zlib.crc32(b"".join(
        member.reader.bytes if hasattr(member, "reader") else member.bytes
        for member in bundle.files.values()
    ))


def update_catalog(catalog: bytes, bundles: list[tuple[bytes, bytes]]) -> bytes:
    replacements = []
    for original, installed in bundles:
        old = bundle_crc(original).to_bytes(4, "little")
        new = bundle_crc(installed).to_bytes(4, "little")
        if old == new:
            continue
        if old == b"\0" * 4 or catalog.count(old) != 1:
            raise ValueError("Expected exactly one nonzero original bundle CRC in catalog")
        replacements.append((catalog.index(old), new))
    if len({offset for offset, _ in replacements}) != len(replacements):
        raise ValueError("Bundle CRC locations overlap")
    for offset, new in replacements:
        catalog = catalog[:offset] + new + catalog[offset + 4:]
    return catalog


def main() -> None:
    import bsdiff4
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--original-root", type=Path, required=True,
                        help="Pristine game tree with paths matching the manifest")
    parser.add_argument("--patched-root", type=Path, required=True,
                        help="Final patched game tree with paths matching the manifest")
    parser.add_argument("--database-output", type=Path,
                        help="Use the freshly built database bundle as the patched database input")
    parser.add_argument("--version", help="New patch version")
    parser.add_argument("--release-revision", type=int, help="New catalog release revision")
    args = parser.parse_args()

    manifest_path = HERE / "patch.windows.candidate.json"
    previous = json.loads(manifest_path.read_text(encoding="utf-8")) if manifest_path.exists() else {}
    version = args.version or previous.get("version", "0.1.0")
    release_revision = args.release_revision if args.release_revision is not None else previous.get("release_revision", 1)
    deltas = HERE / "deltas"
    deltas.mkdir(exist_ok=True)
    files = []
    bundle_pairs = []
    for index, relative in enumerate(FILES):
        source = rooted(args.original_root, relative).read_bytes()
        target_path = args.database_output if index == 0 and args.database_output else rooted(args.patched_root, relative)
        target = target_path.read_bytes()
        if index in (0, 1):
            bundle_pairs.append((source, target))
        if index == 3:
            target = update_catalog(source, bundle_pairs)
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
        "version": version,
        "release_revision": release_revision,
        "filename": f"moomin-midsummer-madness-he-{version}-windows.zip",
        "platforms": {"windows": {"files": files}},
    }
    manifest_path.write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(f"Wrote {len(files)} verified bsdiff files and patch.windows.candidate.json")


if __name__ == "__main__":
    main()

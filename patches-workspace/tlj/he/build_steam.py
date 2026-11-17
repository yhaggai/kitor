"""Build the Steam Hebrew candidate for depot 6311, manifest 404429144808164178."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


HERE = Path(__file__).resolve().parent
PACKAGER = HERE.parents[2] / "kitor/tools/build_patch_packages.py"
VERSION = "catalog-20261025-steam1"


def add_reconstruction(package, stage, target, size, offset, value, original, patched):
    recipe_path = f"recipes/{target}.json"
    recipe = {
        "schema_version": 1,
        "input_sha256": original,
        "output_size": size,
        "operations": [
            {"offset": 0, "length": offset},
            {"hex": value},
            {"offset": offset + 1, "length": size - offset - 1},
        ],
    }
    recipe_bytes = (json.dumps(recipe, indent=2) + "\n").encode()
    (stage / recipe_path).write_bytes(recipe_bytes)
    package["files"].append({
        "path": target,
        "operation": "modify",
        "original_sha256": original,
        "installed_sha256": patched,
        "source": {
            "kind": "reconstruct",
            "input": target,
            "path": recipe_path,
            "sha256": hashlib.sha256(recipe_bytes).hexdigest(),
        },
    })


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    package = json.loads((HERE / "patch.json").read_bytes())
    package["version"] = VERSION
    package["filename"] = f"tlj-he-{VERSION}.zip"
    chapters = next(entry for entry in package["files"] if entry["path"] == "chapters.ini")
    chapters["original_sha256"] = (
        "1aee5ccf4a8a4928195238791780d35aebcc6217fe11908f708bfb58a7db1892"
    )

    with tempfile.TemporaryDirectory(prefix="tlj-steam-package-") as temp:
        stage = Path(temp)
        shutil.copytree(HERE / "payload", stage / "payload", symlinks=True)
        (stage / "recipes").mkdir()

        # The Hebrew Stark detector expects this archive signature.
        add_reconstruction(
            package, stage, "x.xarc", 3032, 22, "01",
            "2b1545a9ac40493f9e16ddb19522ddc57c2134e5af77eb39465d8c21352731f8",
            "e8f217666a04cf4df537df901f350a5bf716bcd22256dac5aa88f1f76b6693f0",
        )
        # Hide the unreadable bitmap from ScummVM so it uses its dialog fallback.
        add_reconstruction(
            package, stage, "game.exe", 95744, 78432, "94",
            "14ecbd33415cf95bccf3e76d563e02843ab5dbbe0efc0d550827f27e681c00d3",
            "d349a0d664a37425d3ba9288cc0ebf58a52f4fd9b9b45d67cdae3ed19d8e0716",
        )

        manifest = stage / "patch.json"
        manifest.write_text(json.dumps(package, indent=2) + "\n", encoding="utf-8")
        subprocess.run(
            [sys.executable, str(PACKAGER), str(manifest), "--universal", "--output", str(args.output)],
            check=True,
        )


if __name__ == "__main__":
    main()

"""Select lightweight CI for Drifter Hebrew subtitle corrections."""

import csv
import hashlib
import io
import json
from pathlib import Path
import re
import subprocess
import sys

MANIFEST = "patches-workspace/drifter/he/patch.json"
PAYLOAD = "patches-workspace/drifter/he/payload/translation.csv"
HEADER = ["Character", "ID", "File", "Context", "English", "Hebrew"]
CONTROL = re.compile(r"\{[^}]*\}|<[^>]*>|%[sdif]")


def from_base(base, path):
    return subprocess.check_output(["git", "show", f"{base}:{path}"])


def validate_csv(before, after):
    def rows(data):
        return list(csv.reader(io.StringIO(data.decode("utf-8"), newline=""), strict=True))

    old, new = rows(before), rows(after)
    if not old or old[0] != HEADER or old[:3] != new[:3] or len(old) != len(new):
        return False
    for number, (previous, current) in enumerate(zip(old[3:], new[3:]), 4):
        if len(previous) != 6 or len(current) != 6 or previous[:5] != current[:5]:
            return False
        if previous[5] != current[5] and (
            not current[5].strip()
            or "\0" in current[5]
            or sorted(CONTROL.findall(previous[5])) != sorted(CONTROL.findall(current[5]))
        ):
            raise ValueError(f"Invalid Hebrew text or changed formatting tokens on record {number}")

    return True


def translation_only(base):
    changed = subprocess.check_output(
        ["git", "diff", "--name-only", "-z", base, "HEAD"]
    ).decode().split("\0")
    if set(filter(None, changed)) != {MANIFEST, PAYLOAD}:
        return False
    for path in (MANIFEST, PAYLOAD):
        if subprocess.check_output(["git", "diff", "--summary", base, "HEAD", "--", path]):
            return False
    old = json.loads(from_base(base, MANIFEST))
    new = json.loads(Path(MANIFEST).read_bytes())
    def source_entries(manifest):
        return [
            item["source"]
            for item in manifest["files"]
            if item.get("source", {}).get("path") == "payload/translation.csv"
        ]

    original_sources, updated_sources = source_entries(old), source_entries(new)
    if len(original_sources) != 1 or len(updated_sources) != 1:
        return False
    expected = updated_sources[0].get("sha256")
    updated_sources[0]["sha256"] = original_sources[0]["sha256"]
    if json.dumps(old, sort_keys=True) != json.dumps(new, sort_keys=True):
        return False

    payload = Path(PAYLOAD).read_bytes()
    if hashlib.sha256(payload).hexdigest() != expected:
        raise ValueError("Translation checksum does not match patch.json")
    return validate_csv(from_base(base, PAYLOAD), payload)


if __name__ == "__main__":
    try:
        print("true" if translation_only(sys.argv[1]) else "false")
    except (ValueError, KeyError, TypeError, csv.Error) as error:
        sys.exit(str(error))

"""Validate text-only Hebrew translation proposals without building installers."""

import csv
import hashlib
import io
import json
from pathlib import Path, PurePosixPath
import re
import subprocess
import sys

HEADER = ["Character", "ID", "File", "Context", "English", "Hebrew"]
CONTROL = re.compile(r"\{[^}]*\}|<[^>]*>|%[sdif]")


def git(*args):
    return subprocess.check_output(["git", *args])


def from_base(base, path):
    return git("show", f"{base}:{path}")


def check_text(previous, current):
    if previous != current and (
        (previous.strip() and not current.strip())
        or "\0" in current
        or sorted(CONTROL.findall(previous)) != sorted(CONTROL.findall(current))
    ):
        raise ValueError("Invalid translation text or changed formatting tokens")


def validate_csv(before, after):
    def rows(data):
        return list(csv.reader(io.StringIO(data.decode("utf-8"), newline=""), strict=True))

    old, new = rows(before), rows(after)
    if not old or old[0] != HEADER or old[:3] != new[:3] or len(old) != len(new):
        return False
    for previous, current in zip(old[3:], new[3:]):
        if len(previous) != 6 or len(current) != 6 or previous[:5] != current[:5]:
            return False
        check_text(previous[5], current[5])
    return True


def json_object(pairs):
    result = dict(pairs)
    if len(result) != len(pairs):
        raise ValueError("Duplicate JSON keys")
    return result


def parse_json(data):
    return json.loads(data, object_pairs_hook=json_object)


def validate_dictionary(before, after):
    old = parse_json(before) if before is not None else {}
    new = parse_json(after)
    if not all(isinstance(data, dict) and all(isinstance(v, str) for v in data.values())
               for data in (old, new)):
        return False
    if not old.keys() <= new.keys():
        return False
    for key, value in new.items():
        check_text(old.get(key, key), value)
    return True


def validate_catalog(before, after):
    old, new = parse_json(before), parse_json(after)
    if old.keys() != new.keys() or len(old["entries"]) != len(new["entries"]):
        return False
    for previous, current in zip(old["entries"], new["entries"]):
        check_text(previous["hebrew"], current["hebrew"])
        for field in ("hebrew", "status", "translator_note"):
            if field in current and not isinstance(current[field], str):
                return False
            previous.pop(field, None)
            current.pop(field, None)
    return old == new


def validator(relative):
    if relative == "payload/translation.csv":
        return validate_csv
    if relative == "translation-catalog.json":
        return validate_catalog
    if relative == "translations.json" or re.fullmatch(
        r"(?:source/)?translations/(?:[\w.-]+/)*[\w.-]+\.json", relative
    ):
        return validate_dictionary
    return None


def validate_manifests(base, directory, payloads, changed_manifests):
    checked = set()
    for path in sorted(Path(directory).glob("patch*.json")):
        name = path.as_posix()
        manifest = parse_json(path.read_bytes())
        affected = [
            entry for entry in manifest.get("files", [])
            if entry.get("source", {}).get("path") in payloads
        ]
        if not affected and name not in changed_manifests:
            continue
        if not affected or name not in changed_manifests:
            raise ValueError(f"Missing matching translation or checksum update in {path}")
        previous = parse_json(from_base(base, name))
        for entry in affected:
            source = entry["source"]
            relative = source["path"]
            old_digest, digest = payloads[relative]
            if source.get("kind") != "copy":
                return False
            if source.get("sha256") != digest:
                raise ValueError(f"Translation checksum does not match {path}: {relative}")
            source["sha256"] = old_digest
            if "installed_sha256" in entry:
                if entry["installed_sha256"] != digest:
                    raise ValueError(f"Installed translation checksum does not match {path}: {relative}")
                entry["installed_sha256"] = old_digest
            checked.add(relative)
        if json.dumps(previous, sort_keys=True) != json.dumps(manifest, sort_keys=True):
            return False
    return checked == payloads.keys()


def translation_only(base):
    changes = git("diff", "--name-status", "--no-renames", "-z", base, "HEAD").decode().split("\0")
    manifests = {}
    translations = {}
    for status, name in zip(changes[::2], changes[1::2]):
        path = PurePosixPath(name)
        parts = path.parts
        if (len(parts) < 4 or parts[0] != "patches-workspace"
                or not re.fullmatch(r"he(?:-[\w-]+)?", parts[2]) or status not in ("A", "M")):
            return False
        # Keep binaries, symlinks and executable files on the full CI path.
        if not git("ls-tree", "HEAD", "--", name).startswith(b"100644 blob "):
            return False
        directory = "/".join(parts[:3])
        relative = "/".join(parts[3:])
        if status == "M" and re.fullmatch(r"patch(?:\.[\w-]+)*\.json", relative):
            manifests.setdefault(directory, set()).add(name)
            continue
        validate = validator(relative)
        if not validate or (status == "A" and validate is not validate_dictionary):
            return False
        translations.setdefault(directory, []).append((status, name, relative, validate))
    if not translations or manifests.keys() - translations.keys():
        return False

    for directory, files in translations.items():
        payloads = {}
        for status, name, relative, validate in files:
            before = from_base(base, name) if status == "M" else None
            after = Path(name).read_bytes()
            if not validate(before, after):
                return False
            if relative.startswith("payload/"):
                payloads[relative] = (hashlib.sha256(before).hexdigest(), hashlib.sha256(after).hexdigest())
        if (payloads or directory in manifests) and not validate_manifests(
            base, directory, payloads, manifests.get(directory, set())
        ):
            return False
    return True


if __name__ == "__main__":
    try:
        print("true" if translation_only(sys.argv[1]) else "false")
    except (ValueError, KeyError, TypeError, csv.Error) as error:
        sys.exit(str(error))

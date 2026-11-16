"""Generate the Drifter's display-order payload from the logical Hebrew CSV."""
import argparse
import csv
import hashlib
import io
import json
import re
from pathlib import Path

from bidi.algorithm import get_display

ROOT = Path(__file__).resolve().parent
CATALOG = ROOT / "translation-catalog.csv"
PAYLOAD = ROOT / "payload/translation.csv"
MANIFEST = ROOT / "patch.json"
BUILD_RECEIPT = ROOT / "build-source.json"
HEADER = ["Character", "ID", "File", "Context", "English", "Hebrew"]
PREFIX = re.compile(r'(?:(?:"(?:[^"]|"")*"|[^,"\r\n]*),){5}')
HEBREW = re.compile(r"[\u05d0-\u05ea]")
PROTECTED = re.compile(r"\{[^{}\r\n]+\}|<[^<>\r\n]+>|%[sdif]|[0-9]+(?:\.[0-9]+)? CPM\b")
# This existing mixed-direction label has two valid bidi interpretations.
# Preserve its shipped spelling and layout when its translation is unchanged.
LEGACY_LABEL = ("גופן הדיאלוג ב-HD: {0}", "{0} :HD-ב גולאידה ןפוג")


def reorder_line(line):
    if not HEBREW.search(line):
        return line
    protected = {}

    def hold(match):
        marker = chr(0xE000 + len(protected))
        if marker in line or len(protected) >= 0x1900:
            raise ValueError("Reserved character in Drifter translation")
        protected[marker] = match[0]
        return marker

    displayed = get_display(PROTECTED.sub(hold, line), base_dir="R")
    return "".join(protected.get(char, char) for char in displayed)


def convert(source):
    # CSV parsing tracks physical lines so untouched cells and record separators
    # retain their bytes, including quoted newlines and the final line without EOL.
    lines = re.findall(r"[^\r\n]*(?:\r\n|\n|\r)|[^\r\n]+$", source)
    reader = csv.reader(io.StringIO(source, newline=""), strict=True)
    if next(reader, None) != HEADER:
        raise ValueError("Unexpected Drifter CSV header")
    cursor = reader.line_num
    output = [source[:len(lines[0])]]
    logical_label, display_label = LEGACY_LABEL
    for index, row in enumerate(reader, start=1):
        raw = "".join(lines[cursor:reader.line_num])
        cursor = reader.line_num
        if len(row) != len(HEADER):
            raise ValueError(f"Unexpected Drifter CSV record width at record {index + 1}")
        text = row[-1]
        if index < 3 or not HEBREW.search(text):
            output.append(raw)
            continue
        if text == logical_label:
            converted = display_label
        else:
            converted = "".join(
                line if line in ("\r\n", "\n", "\r") else reorder_line(line)
                for line in re.split(r"(\r\n|\n|\r)", text)
            )
        if converted == text:
            output.append(raw)
            continue
        prefix = PREFIX.match(raw)
        if prefix is None:
            raise ValueError(f"Could not locate translation cell at record {index + 1}")
        ending = re.search(r"(?:\r\n|\n|\r)$", raw)
        if raw[prefix.end():].startswith('"') or any(c in converted for c in ',\"\r\n'):
            converted = '"' + converted.replace('"', '""') + '"'
        output.extend((prefix[0], converted, ending[0] if ending else ""))
    if cursor != len(lines):
        raise ValueError("Incomplete Drifter CSV conversion")
    return "".join(output)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Verify the generated payload and manifest hash")
    args = parser.parse_args()
    source = CATALOG.read_bytes()
    data = convert(source.decode("utf-8")).encode("utf-8")
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    asset, = (entry["source"] for entry in manifest["files"] if entry["path"] == "translation.csv")
    digest = hashlib.sha256(data).hexdigest()
    receipt = json.dumps({
        "source_sha256": hashlib.sha256(source).hexdigest(),
        "payload_sha256": digest,
    }, indent=2) + "\n"
    if args.check:
        if (PAYLOAD.read_bytes() != data or asset["sha256"] != digest
                or BUILD_RECEIPT.read_bytes() != receipt.encode("utf-8")):
            raise SystemExit("Drifter payload is out of date. Run patches-workspace/drifter/he/build.py.")
    else:
        PAYLOAD.write_bytes(data)
        BUILD_RECEIPT.write_bytes(receipt.encode("utf-8"))
        if asset["sha256"] != digest:
            asset["sha256"] = digest
            MANIFEST.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(f"Drifter display-order CSV SHA-256: {digest}")


if __name__ == "__main__":
    main()

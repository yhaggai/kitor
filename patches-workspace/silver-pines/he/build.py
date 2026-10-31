"""Build the Hebrew English-slot bundle from a pristine Silver Pines bundle and catalog."""
import argparse
from collections import Counter
import json
from pathlib import Path
import re
import struct
import textwrap

import UnityPy
from bidi import algorithm as bidi

TOKEN = re.compile(r'(<[^<>\n]{1,100}>|\{[^{}\n]+\})')
WIDTH = {"Dialogue": 44, "Radio": 44, "Television": 44, "Video": 44, "Items": 39,
         "Lore": 52, "PauseHints": 48, "Examinable": 44, "Objectives": 45}


def visual(text, table):
    if not re.search("[א-ת]", text):
        return text
    text = text.translate(str.maketrans({"׳": "'", "״": '"', "־": "-"}))
    styles, stack, symbols, plain = [], [], {}, []
    for token in TOKEN.split(text):
        if token.startswith("</") and stack:
            stack.pop()
        elif token.startswith("<") and not token.startswith("<sprite"):
            stack.append(token)
        else:
            if TOKEN.fullmatch(token):
                symbol = chr(0xe000 + len(symbols))
                symbols[symbol] = token
                token = symbol
            plain.extend(token)
            styles.extend([tuple(stack)] * len(token))
    result, offset = [], 0
    for paragraph in "".join(plain).split("\n"):
        lines = textwrap.wrap(paragraph, width=WIDTH.get(table, 48), replace_whitespace=False,
                              drop_whitespace=False, break_long_words=False, break_on_hyphens=False) or [""]
        for line in lines:
            state = bidi.get_empty_storage()
            state.update(base_level=1, base_dir="R")
            bidi.get_embedding_levels(line, state)
            for index, char in enumerate(state["chars"]):
                char["style"] = styles[offset + index]
            bidi.explicit_embed_and_overrides(state)
            bidi.resolve_weak_types(state)
            bidi.resolve_neutral_types(state, False)
            bidi.resolve_implicit_levels(state, False)
            bidi.reorder_resolved_levels(state, False)
            bidi.apply_mirroring(state, False)
            output, active = [], ()
            for char in state["chars"]:
                style = char["style"]
                common = 0
                while common < min(len(active), len(style)) and active[common] == style[common]:
                    common += 1
                for tag in reversed(active[common:]):
                    output.append("</" + re.match(r"<(\w+)", tag)[1] + ">")
                output.extend(style[common:])
                output.append(symbols.get(char["ch"], char["ch"]))
                active = style
            for tag in reversed(active):
                output.append("</" + re.match(r"<(\w+)", tag)[1] + ">")
            result.append("".join(output).strip())
            offset += len(line)
        offset += 1
    return "\n".join(result)


def entries(raw):
    def uint(offset):
        return struct.unpack_from("<I", raw, offset)[0]
    cursor = 32 + (uint(28) + 3) // 4 * 4
    cursor += 4 + (uint(cursor) + 3) // 4 * 4 + 12
    cursor += 4 + 8 * uint(cursor)
    count = uint(cursor)
    cursor += 4
    for _ in range(count):
        key = str(struct.unpack_from("<q", raw, cursor)[0])
        start = cursor + 8
        length = uint(start)
        end = start + 4 + (length + 3) // 4 * 4
        yield key, start, end, raw[start+4:start+4+length].decode()
        cursor = end + 4 + 8 * uint(end)


def build(original, catalog, destination):
    tables = json.loads(catalog.read_text(encoding="utf8"))
    env = UnityPy.load(original.read_bytes())
    expected = {}
    for obj in env.objects:
        raw = obj.get_raw_data()
        if obj.type.name == "MonoBehaviour":
            table = obj.peek_name().removesuffix("_en")
            records = tables.pop(table)
            chunks, cursor = [], 0
            for key, start, end, source in entries(raw):
                record = records.pop(key)
                if record["en"] != source:
                    raise ValueError(f"Changed source {table}/{key}")
                display = visual(record["he"], table)
                tokens = r"\{[^{}]+\}|<sprite [^>]+>"
                if Counter(re.findall(tokens, source)) != Counter(re.findall(tokens, display)):
                    raise ValueError(f"Lost control token {table}/{key}")
                data = display.encode()
                chunks.extend((raw[cursor:start], struct.pack("<I",len(data)), data, bytes(-len(data)%4)))
                cursor = end
            assert not records
            raw = b"".join(chunks) + raw[cursor:]
            obj.set_raw_data(raw)
        expected[obj.path_id] = raw
    assert not tables
    blob = env.file.save(packer="lz4")
    assert {o.path_id:o.get_raw_data() for o in UnityPy.load(blob).objects} == expected
    destination.write_bytes(blob)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    for arg in ("original", "catalog", "destination"):
        parser.add_argument(arg, type=Path)
    args = parser.parse_args()
    build(args.original, args.catalog, args.destination)

"""Rename the displayed language while keeping the game's existing RTL slot."""
import struct

import UnityPy


def unity_string(value: str) -> bytes:
    encoded = value.encode("utf-8")
    return struct.pack("<I", len(encoded)) + encoded + bytes(-len(encoded) % 4)


def replace_language_display_name(source: bytes) -> bytes:
    env = UnityPy.load(source)
    old = unity_string("العربية")
    new = unity_string("עברית")
    marker = unity_string("LanguagesNamesLockit")
    matches = [
        (obj, raw)
        for obj in env.objects
        if obj.type.name == "MonoBehaviour"
        and marker in (raw := obj.get_raw_data())
        and old in raw
    ]
    if len(matches) != 1 or matches[0][1].count(old) != 1:
        raise ValueError("Expected one Arabic display name in LanguagesNamesLockit")

    obj, original = matches[0]
    updated = original.replace(old, new, 1)
    obj.set_raw_data(updated)
    result = env.file.save()

    reopened = {item.path_id: item.get_raw_data() for item in UnityPy.load(result).objects}
    if len(reopened) != len(env.objects) or any(
        reopened[item.path_id] != (updated if item.path_id == obj.path_id else item.get_raw_data())
        for item in env.objects
    ):
        raise ValueError("Unexpected changes after saving language display names")
    return result

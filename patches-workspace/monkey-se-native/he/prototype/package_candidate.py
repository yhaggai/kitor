"""Build local, hash-guarded reconstruction recipes from owned game resources.

The generated native patches are experimental; do not publish without reviewing
translation and game-derived font redistribution rights.
"""
import argparse
import hashlib
import json
import mmap
from pathlib import Path
import struct
import os
import tempfile

ROOT = Path(__file__).resolve().parents[4]
def copy(ops, offset, length):
    if not length:
        return
    if ops and 'offset' in ops[-1] and ops[-1]['offset'] + ops[-1]['length'] == offset:
        ops[-1]['length'] += length
    else:
        ops.append(dict(offset=offset, length=length))


def delta(original, translated, base):
    translated = bytes(translated)
    block = 16
    index = {}
    for p in range(0, len(original) - block + 1, block):
        index.setdefault(original[p:p + block], p)
    ops, pending, p = [], bytearray(), 0
    while p < len(translated):
        at = index.get(translated[p:p + block]) if p + block <= len(translated) else None
        if at is None:
            pending.append(translated[p])
            p += 1
            continue
        if pending:
            ops.append(dict(hex=pending.hex()))
            pending.clear()
        n = block
        while at + n < len(original) and p + n < len(translated) and original[at + n] == translated[p + n]:
            n += 1
        copy(ops, base + at, n)
        p += n
    if pending:
        ops.append(dict(hex=pending.hex()))
    return ops


HELPER = {'copy': copy, 'delta': delta}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def save(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(mode='w', encoding='utf-8', newline='\n', dir=path.parent, delete=False) as stream:
        stream.write(json.dumps(value, ensure_ascii=False, separators=(',', ':')) + '\n')
        temporary = stream.name
    os.replace(temporary, path)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input', type=Path, required=True)
    parser.add_argument('--pak', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--classic', action='store_true', help='Include verified classic-mode resources and synchronized lookup keys')
    a = parser.parse_args()
    report = json.loads((a.input / 'build-report.json').read_text())
    if a.classic:
        report['classic'] = json.loads((a.input / 'classic-candidate/report.json').read_text())
        report['limitations'] = ['Unmatched dialogue and some interface text remain English',
                                'Full-game playback and RTL paragraph wrapping are not certified',
                                'Old saves can retain script positions from earlier resource versions']
    game = report['game']
    game_id = 'monkey-se-native' if game == 1 else 'monkey2-se-native'
    files = []
    recipes = []

    def add(name, source, target, operations):
        recipe_name = f'recipes/{len(recipes)}.json'
        recipe = dict(schema_version=1, input_sha256=digest(source), output_size=len(target), operations=operations)
        path = a.output / recipe_name
        save(path, recipe)
        if path.stat().st_size > 16 * 1024 * 1024:
            raise ValueError('Recipe exceeds runtime limit')
        recipes.append(recipe_name)
        files.append(dict(path=name, operation='modify', original_sha256=digest(source),
                          installed_sha256=digest(target), source=dict(kind='reconstruct',
                          path=recipe_name, sha256=digest(path.read_bytes()), input=name)))

    with a.pak.open('rb') as handle, mmap.mmap(handle.fileno(), 0, access=mmap.ACCESS_READ) as source:
        h = struct.unpack_from('<4s9I', source)
        _, _, _, entries, names, data_start, _, entry_size, names_size, _ = h
        filenames = source[names:names + names_size].split(b'\0')
        header = bytearray(source[:data_start])
        chunks = []
        appended = 0
        for i, raw in enumerate(filenames[:-1]):
            name = raw.decode()
            payload = a.input / 'payload' / name
            if a.classic and (a.input / 'classic-candidate' / name).is_file():
                payload = a.input / 'classic-candidate' / name
            # Only the French localized speech is overridden for MI2.
            if not payload.is_file() or (game == 2 and name == 'localization/en.speech.info'):
                continue
            off, _, packed, size, flag = struct.unpack_from('<5I', source, entries + i * 20)
            assert flag == 0 and packed == size
            modified = payload.read_bytes()
            new_offset = len(source) + appended - data_start
            struct.pack_into('<I', header, entries + i * 20, new_offset)
            struct.pack_into('<II', header, entries + i * 20 + 8, len(modified), len(modified))
            chunks.append((data_start + off, size, modified))
            appended += len(modified)
        struct.pack_into('<I', header, 36, len(source) + appended - data_start)
        ops = HELPER['delta'](source[:data_start], header, 0)
        HELPER['copy'](ops, data_start, len(source) - data_start)
        # Stream output to disk; its only purpose is verification and local tests.
        out = a.input / 'candidate.pak'
        with out.open('wb') as target:
            target.write(header)
            for start in range(data_start, len(source), 1024 * 1024):
                target.write(source[start:min(start + 1024 * 1024, len(source))])
            for offset, size, modified in chunks:
                ops.extend(HELPER['delta'](source[offset:offset + size], modified, offset))
                target.write(modified)
        with out.open('rb') as stream, mmap.mmap(stream.fileno(), 0, access=mmap.ACCESS_READ) as target:
            add(a.pak.name, source, target, ops)
    for name in (['audio/speech.info', 'localization/uiText.info'] if game == 1 else []):
        source = (a.input / 'original' / name).read_bytes()
        target = (a.input / 'payload' / name).read_bytes()
        if a.classic and (a.input / 'classic-candidate' / name).is_file():
            target = (a.input / 'classic-candidate' / name).read_bytes()
        add(name, source, target, HELPER['delta'](source, target, 0))
    # MI2 must retain its original executable: display-string edits cause early exit.
    if game == 1:
        from build_candidate import encode
        exe = 'MISE.exe' if game == 1 else 'Monkey2.exe'
        original_exe = (a.pak.parent / exe).read_bytes()
        renamed_exe = bytearray(original_exe)
        for before, after in [(b'Fran\xe7ais\0', encode('עברית') + b'\0'),
                              (b'S\xe9lectionnez votre langue :\0', encode('בחירת שפה:') + b'\0')]:
            assert original_exe.count(before) == 1, 'Unsupported executable language labels'
            assert len(after) <= len(before)
            offset = original_exe.index(before)
            renamed_exe[offset:offset + len(before)] = after.ljust(len(before), b'\0')
        # Only two fixed-size display strings change. Instructions and imports stay intact.
        add(exe, original_exe, renamed_exe, delta(original_exe, renamed_exe, 0))
        (a.input / 'payload' / exe).write_bytes(renamed_exe)
    manifest = dict(schema_version=1, release_revision=1 if a.classic else 0, scummvm_id=game_id, language='he',
                    version='0.2.0-native-candidate' if a.classic else '0.1.0-native-candidate', filename=f'{game_id}-he.zip',
                    platforms={'windows': {}}, files=files)
    save(a.output / 'patch.json', manifest)
    save(a.output / 'coverage.json', report)
    save(a.output / 'provenance.json', dict(release_ready=False, status='experimental',
         translation='https://github.com/adventurebrew/scummworks',
         speech_ids='https://github.com/jmnunezizu/scummkit/tree/main/third_party/ultimate-talkie',
         formats='https://github.com/ShadowNate/MISETranslator',
         runtime='Native Windows Special Edition, French language slot',
         limitations=report['limitations'], redistribution='Local candidate; not approved for publication'))
    print(game_id, 'recipes:', len(recipes), 'bytes:', sum((a.output / p).stat().st_size for p in recipes))


if __name__ == '__main__':
    main()

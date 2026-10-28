"""Create an isolated F1 test copy from an existing remastered Hebrew test copy."""
import argparse
from pathlib import Path
import shutil
import struct


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--game', type=int, choices=(1, 2), required=True)
    a = parser.parse_args()
    root = Path(__file__).resolve().parents[4]
    source = root / f'dist/monkey-native-tests/mi{a.game}-packaged'
    target = root / f'dist/monkey-native-tests/mi{a.game}-f1'
    candidate = Path(__file__).parent / f'generated/mi{a.game}/classic-candidate'
    assert (candidate / 'report.json').is_file()
    if not target.exists():
        shutil.copytree(source, target)
    pak = 'Monkey1.pak' if a.game == 1 else 'monkey2.pak'
    shutil.copy2(source / pak, target / pak)
    with (target / pak).open('r+b') as stream:
        h = struct.unpack('<4s9I', stream.read(40))
        stream.seek(h[3]); records = list(struct.iter_unpack('<5I', stream.read(h[7])))
        stream.seek(h[4]); names = stream.read(h[8]).split(b'\0')
        for i, (raw, record) in enumerate(zip(names, records)):
            path = candidate / raw.decode()
            if not path.is_file():
                continue
            content = path.read_bytes()
            stream.seek(0, 2); offset = stream.tell() - h[5]
            stream.write(content)
            stream.seek(h[3] + i * 20)
            stream.write(struct.pack('<5I', offset, record[1], len(content), len(content), 0))
        stream.seek(0, 2); data_size = stream.tell() - h[5]
        stream.seek(36); stream.write(struct.pack('<I', data_size))
    if a.game == 1:
        shutil.copy2(candidate / 'audio/speech.info', target / 'audio/speech.info')
    print(target)


if __name__ == '__main__':
    main()

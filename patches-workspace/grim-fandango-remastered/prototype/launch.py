"""Launch an isolated patched or unpatched ScummVM comparison on either OS."""
import argparse
import configparser
from pathlib import Path
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--scummvm', type=Path, required=True)
    parser.add_argument('--game', type=Path, required=True)
    parser.add_argument('--baseline', action='store_true')
    args = parser.parse_args()
    game = args.game.resolve()
    executable = args.scummvm.resolve()
    if not executable.is_file() or not (game / 'grim.en.tab').is_file():
        parser.error('Supply an existing ScummVM executable and game resource directory containing grim.en.tab.')
    name = 'baseline' if args.baseline else 'hebrew'
    runtime = Path(__file__).resolve().parent / 'runtime'
    saves = runtime / (name + '-saves')
    saves.mkdir(parents=True, exist_ok=True)
    config = configparser.ConfigParser(interpolation=None)
    config['scummvm'] = {'fullscreen': 'false', 'subtitles': 'true', 'speech_mute': 'false'}
    config['keymapper'] = {'keymap_global_DEBUGGER': 'F9'}
    config['grim-prototype'] = {
        'description': 'Grim Fandango Remastered - ' + name,
        'engineid': 'grim', 'gameid': 'grim', 'extra': 'Remastered',
        'path': str(game), 'savepath': str(saves), 'language': 'en',
        'platform': 'windows', 'grim_language': '0', 'datausr_load': 'false',
        'subtitles': 'true', 'speech_mute': 'false', 'speech_volume': '192',
        'music_volume': '192', 'sfx_volume': '192', 'directors_commentary': '0',
    }
    config_path = runtime / (name + '.ini')
    with config_path.open('w', encoding='utf-8') as stream:
        config.write(stream)
    raise SystemExit(subprocess.call([
        str(executable), '--config=' + str(config_path),
        '--logfile=' + str(runtime / (name + '.log')), 'grim-prototype',
    ]))


if __name__ == '__main__':
    main()

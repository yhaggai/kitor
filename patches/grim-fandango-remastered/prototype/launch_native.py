"""Run the Windows native experiment with its own saves and settings."""
import argparse
import os
from pathlib import Path
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--game', type=Path, default=Path(__file__).resolve().parent / 'runtime/game')
    parser.add_argument('--profile', type=Path, default=Path(__file__).resolve().parent / 'runtime/native-profile')
    args = parser.parse_args()
    game = args.game.resolve()
    executable = game / 'GrimFandango.exe'
    if os.name != 'nt':
        parser.error('This launcher requires the Windows native game.')
    if not executable.is_file() or not (game / 'grim.en.tab').is_file():
        parser.error('Game directory must contain GrimFandango.exe and grim.en.tab.')
    profile = args.profile.resolve()
    profile.mkdir(parents=True, exist_ok=True)
    environment = os.environ.copy()
    environment['APPDATA'] = str(profile)
    # The native executable reads APPDATA for Doublefine/GrimFandango saves.
    # Working directory selects the copied resources; -log enables Grim.log.
    return subprocess.call([str(executable), '-log'], cwd=game, env=environment)


if __name__ == '__main__':
    raise SystemExit(main())

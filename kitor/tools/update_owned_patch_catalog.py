#!/usr/bin/env python3
"""Index the reviewed owned catalog entries without enabling unverified installs."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
WORKSPACE = ROOT / 'patches-workspace'
ENTRIES = {
    1: 'simon1-25th', 4: 'monkey-se', 5: 'monkey2-se', 16: 'pq1',
    17: 'sq1sci', 18: 'sq3', 19: 'torin', 22: 'stanley-parable',
    27: 'kyra1-cd', 36: 'tesla-effect', 39: 'she-sees-red', 68: 'spyfox',
    69: 'pajama', 72: 'sq5', 75: 'ethan-carter', 76: 'toonstruck',
    77: 'life-is-strange', 87: 'portal2', 90: 'gob2', 97: 'shivah',
    99: 'journey-down1', 107: 'tlj', 112: 'sky', 121: 'kathy-rain-dc',
    130: 'sq2', 136: 'sq1', 160: 'drifter', 164: 'gemini-rue',
}


def main():
    rows = []
    for index, game in ENTRIES.items():
        folder = WORKSPACE / game / 'he'
        source = folder / 'provenance.json'
        data = json.loads(source.read_text())
        if data['catalog_index'] != index:
            raise ValueError(f'Catalog ID mismatch: {source}')
        manifests = [p for p in (folder / 'patch.json', folder / 'patch.windows.candidate.json') if p.exists()]
        if len(manifests) > 1:
            raise ValueError(f'Ambiguous manifest: {folder}')
        manifest = json.loads(manifests[0].read_text()) if manifests else None
        artifacts = []
        if manifest:
            for platform in ('windows', 'osx'):
                artifact = ROOT / 'dist' / 'patches' / f"{game}-he-{manifest['version']}-{platform}.zip"
                if platform in manifest['platforms'] and artifact.exists():
                    digest = hashlib.file_digest(artifact.open('rb'), 'sha256').hexdigest()
                    artifacts.append({'platform': platform, 'path': str(artifact.relative_to(ROOT)),
                                      'size': artifact.stat().st_size, 'sha256': digest})
        rows.append({'catalog_index': index, 'game_id': game, 'title': data['title'],
                     'owned_stores': [s.lower() for s in data['owned_stores']],
                     'directory': str(folder.relative_to(ROOT)),
                     'status': data['status'], 'release_ready': data.get('release_ready', False),
                     'manifest': str(manifests[0].relative_to(ROOT)) if manifests else None,
                     'declared_platforms': list(manifest['platforms']) if manifest else [],
                     'artifacts': artifacts, 'blockers': data.get('blockers', [])})
    result = {'schema_version': 1, 'ownership_checked_at': '2026-10-25',
              'catalog_entry_count': len(rows), 'source_catalog': 'Downloads/json/all_games.json (174 entries)',
              'ownership_sources': {'steam': 'yhaggai signed-in All Games page, 103 entries',
                                    'gog': 'Authenticated account library, 36 titles'},
              'scope': 'Owned patch packages are indexed in Kitor; new packages are published as unverified prereleases. No game installation was modified.',
              'specification': 'patches-workspace/specification.json',
              'build_both_platforms': 'python3 kitor/tools/build_patch_packages.py <patch.json> --output dist/patches',
              'build_windows_candidate': 'python3 kitor/tools/build_patch_packages.py <patch.windows.candidate.json> --platform windows --output dist/patches',
              'verify_lifecycle': 'python3 kitor/tools/run_lifecycle_tests.py --nocapture',
              'entries': rows}
    destination = WORKSPACE / 'owned-catalog.json'
    destination.write_text(json.dumps(result, indent=2, ensure_ascii=False) + '\n')
    labels = {
        'packaged_unverified': 'Windows + macOS packages; gameplay unverified',
        'staged_windows_payload_awaiting_compatibility_validation': 'Windows candidate; macOS pending',
        'experimental_windows_payload_requires_gameplay': 'Experimental Windows candidate; macOS pending',
        'no_patch_required': 'Hebrew built in; no external patch',
        'blocked': 'Owned edition incompatible with catalog patch',
    }
    lines = [
        '# Owned-game patch workspace', '',
        'Ownership snapshot: 25 October 2026. All 28 catalog matches are represented here.', '',
        'Installable packages are indexed in the Kitor registry. New packages are published as unverified prereleases; game-specific Windows/macOS gameplay validation remains outstanding. Some entries need manual selection/detection support, and modern games must be launched through Steam/GOG.', '',
        '## Contract', '',
        'Every releasable patch needs Windows and macOS artifacts, a declared file plan, source hashes, edition/dependency information, and selective install/uninstall through Kitor. Back up only affected existing files; restore those originals and remove patch-created files. Never copy or clear the entire game directory. See [specification.json](specification.json).', '',
        'Portable data packages use Kitor and the appropriate native game/ScummVM runtime; packaging a ZIP does not compile a new native installer. Dependencies and exceptions live in each provenance.json.', '',
        '## Inventory', '', '| # | Game | Owned on | Status |', '|---|---|---|---|',
    ]
    for row in rows:
        relative = row['directory'].removeprefix('patches-workspace/')
        lines.append(f"| {row['catalog_index']} | [{row['title']}]({relative}/provenance.json) | {', '.join(row['owned_stores'])} | {labels.get(row['status'], row['status'])} |")
    lines += ['', '## Build and verification', '',
        'Run from the repository root:', '', '```sh',
        'python3 kitor/tools/build_patch_packages.py patches-workspace/<game>/he/patch.json --output dist/patches',
        'python3 kitor/tools/build_patch_packages.py patches-workspace/<game>/he/patch.windows.candidate.json --platform windows --output dist/patches',
        'python3 -m unittest discover -s kitor/tools -p "test_*.py"',
        'python3 kitor/tools/run_lifecycle_tests.py --nocapture',
        'python3 kitor/tools/update_owned_patch_catalog.py', '```', '',
        'The [machine-readable inventory](owned-catalog.json) records local artifact paths and SHA-256 hashes. The [validation record](validation.json) distinguishes fixture results from native gameplay checks.', '',
        'Journey Down and Portal 2 have payloads exceeding GitHub single-file limits. Those assets are excluded from Git and hydrated from private, SHA256-pinned build-input release assets. CI builds both dual-platform packages and Windows candidates.', '',
        'Release automation requires explicit release_ready=true in provenance.json and validated dual-platform manifests. All imported entries remain false. Raw upstream installers and ownership metadata are excluded from release ZIPs.', '',
    ]
    (WORKSPACE / 'README.md').write_text('\n'.join(lines))
    print(f'Indexed {len(rows)} owned catalog entries: {destination}')


if __name__ == '__main__':
    main()

"""Add the separately launchable native candidates without changing classic entries."""
import copy
import json
from pathlib import Path

root = Path(__file__).resolve().parents[4]
path = root / 'kitor/patches/registry.json'
text = path.read_text(encoding='utf-8')
registry = json.loads(text)
new = []
for classic in ('monkey-se', 'monkey2-se'):
    native = classic + '-native'
    if any(g['scummvm_id'] == native for g in registry['games']):
        continue
    item = copy.deepcopy(next(g for g in registry['games'] if g['scummvm_id'] == classic))
    item['scummvm_id'] = native
    item['title'] += ' (Native Remastered)'
    item['platforms'] = dict(windows=True, osx=False, linux=False)
    item['launch_config'] = dict(launcher='native', args=[])
    item['purchase_guide'] = native + '.md'
    item['patches'] = [dict(language='he', version='0.2.0-native-candidate',
        author='AdventureBrew; native adaptation and menu translation: Kitor',
        download_url=f'bundled://{native}/he', source_type='bundled', checksum=None,
        file_size=None, filename=f'{native}-he.zip', post_install_script=None, install_manifest='patch.windows.candidate.json',
        description='Native Windows Hebrew candidate using the classic AdventureBrew dialogue and new Hebrew menus. The French slot contains Hebrew. F1 classic graphics include Hebrew subtitles and bitmap fonts. Unmatched or conflicting dialogue stays English. Full-game and RTL wrapping validation are incomplete. Requires a supported original edition; installation is reversible. Steam must be running for Monkey Island 2.')]
    new.append(item)
if new:
    marker = '\n  ],\n  "metadata"'
    assert text.count(marker) == 1
    entries = ',\n' + ',\n'.join('\n'.join('    ' + line for line in json.dumps(g, ensure_ascii=False, indent=2).splitlines()) for g in new)
    path.write_text(text.replace(marker, entries + marker), encoding='utf-8')
for classic in ('monkey-se', 'monkey2-se'):
    native = classic + '-native'
    (root / 'kitor/patches/guides' / (native + '.md')).write_text(
        '# Native Special Edition Hebrew\n\nInstall the Windows Special Edition from Steam or GOG, then scan its folder in Kitor. '
        'The native entry runs the remastered graphics directly. The classic ScummVM entry remains separate.\n\n'
        'Install Hebrew. Select **עברית** in MI1 or **French** in Steam for MI2. F1 switches graphics while retaining translated subtitles. '
        'Enable subtitles. Monkey Island 2 requires Steam to be running and a complete installation including steam_api.dll.\n\n'
        'Experimental adaptation: classic dialogue credits remain with AdventureBrew; menus are newly translated. '
        'Unmatched dialogue stays English. Full-game coverage and paragraph wrapping remain under review. '
        'Uninstall in Kitor restores only affected files and preserves saves.\n', encoding='utf-8')

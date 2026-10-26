"""Package an explicit source allowlist. Never include game data or backups."""
from pathlib import Path
import zipfile
ROOT = Path(__file__).resolve().parents[1]
files = ['README.md', 'LICENSE', '.gitignore', 'requirements.txt', 'compatibility.json',
         'catalog-contract.json', 'install-macos.command', 'install-windows.cmd']
for directory, pattern in [('tools', '*.py'), ('translations', 'part-*.json'),
                           ('fonts', '*.json'), ('runtime', '*.cs'),
                           ('patcher', '*.cs'), ('patcher', '*.csproj'),
                           ('docs', '*.md'), ('docs', 'package-validation.json'),
                           ('tests', 'test_*.py'), ('tests/runtime', '*.cs'),
                           ('tests/runtime', '*.csproj'), ('.github/workflows', '*.yml')]:
    files.extend(p.relative_to(ROOT).as_posix() for p in (ROOT / directory).glob(pattern))
files = sorted(set(files))
for name in files:
    if (ROOT / name).is_symlink():
        raise ValueError('Refusing symlink in source package: ' + name)
(ROOT / 'dist').mkdir(exist_ok=True)
for platform in ['macos', 'windows']:
    path = ROOT / 'dist' / f'owlboy-hebrew-0.2.0-beta.2-{platform}-source.zip'
    with zipfile.ZipFile(path, 'w', zipfile.ZIP_DEFLATED) as archive:
        for name in files:
            archive.write(ROOT / name, 'owlboy-hebrew/' + name)
    print(path)

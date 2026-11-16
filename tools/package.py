"""Package the compiled executable, documentation and dependency licenses."""
import json
import pathlib
import shutil
import subprocess

root = pathlib.Path(__file__).resolve().parent.parent
dist = root / 'dist'
dist.mkdir(exist_ok=True)
shutil.copy2(root / 'target/release/wiu.exe', dist / 'wiu.exe')
for name in ['README.md', 'LICENSE']:
    shutil.copy2(root / name, dist / name)
metadata = json.loads(subprocess.check_output(
    ['cargo', 'metadata', '--offline', '--locked', '--format-version', '1'], cwd=root))
lines = ['# Third-party dependencies\n\nVersions and declared licenses, including build and platform dependencies.\n']
for package in sorted(metadata['packages'], key=lambda p: (p['name'], p['version'])):
    if package['name'] == 'wiu':
        continue
    name = package['name'] + '-' + package['version']
    lines.append(f"- {name}: {package.get('license', 'See license files')}\n")
    source = pathlib.Path(package['manifest_path']).parent
    for license_file in source.iterdir():
        if license_file.is_file() and license_file.name.upper().startswith(('LICENSE', 'COPYING', 'NOTICE', 'OFL')):
            destination = dist / 'licenses' / name
            destination.mkdir(parents=True, exist_ok=True)
            shutil.copy2(license_file, destination / license_file.name)
(dist / 'THIRD_PARTY.md').write_text(''.join(lines), encoding='utf-8')
print(f'Packaged {dist / "wiu.exe"}')

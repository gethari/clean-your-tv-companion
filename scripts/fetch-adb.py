"""Fetch official host platform-tools for preview packaging, retaining notices and provenance.
Stable release gate: replace mutable latest with reviewed pinned archive URLs and hashes.
"""
import hashlib
import io
import json
from pathlib import Path
import platform
import urllib.request
import zipfile

host = {'Linux': 'linux', 'Windows': 'windows', 'Darwin': 'darwin'}[platform.system()]
url = f'https://dl.google.com/android/repository/platform-tools-latest-{host}.zip'
root = Path(__file__).resolve().parents[1] / 'src-tauri/resources'
root.mkdir(parents=True, exist_ok=True)
with urllib.request.urlopen(url, timeout=60) as response:
    data = response.read(100 * 1024 * 1024 + 1)
if len(data) > 100 * 1024 * 1024:
    raise SystemExit('Archive exceeds packaging limit')
with zipfile.ZipFile(io.BytesIO(data)) as archive:
    for entry in archive.infolist():
        path = Path(entry.filename)
        if '..' in path.parts or path.is_absolute() or not path.parts or path.parts[0] != 'platform-tools':
            raise SystemExit('Unexpected archive path')
    archive.extractall(root)
    adb = root / 'platform-tools' / ('adb.exe' if host == 'windows' else 'adb')
    if host != 'windows':
        adb.chmod(0o755)
(root / 'platform-tools/provenance.json').write_text(json.dumps({
    'source': url, 'sha256': hashlib.sha256(data).hexdigest(), 'usage': 'unsigned inspection preview'
}, indent=2))
print('Official platform-tools prepared for', host)

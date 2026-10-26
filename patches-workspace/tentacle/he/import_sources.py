#!/usr/bin/env python3
"""Fetch pinned upstream inputs for recipe regeneration; never execute the installer."""
import hashlib,json,subprocess,urllib.request
from pathlib import Path
ROOT=Path(__file__).resolve().parent
sources=json.loads((ROOT/'source-inputs.json').read_text())
for source in sources:
 path=ROOT/'source'/source['filename'];path.parent.mkdir(exist_ok=True)
 if not path.exists():
  with urllib.request.urlopen(source['url']) as response:data=response.read()
  assert hashlib.sha256(data).hexdigest()==source['sha256'],'Downloaded input checksum mismatch'
  path.write_bytes(data)
 assert hashlib.sha256(path.read_bytes()).hexdigest()==source['sha256'],'Existing input checksum mismatch'
subprocess.run(['7z','x','-y','-o'+str(ROOT/'source/extracted'),str(ROOT/'source/DOTT-Heb-AdventureBrew-BETA2.exe'),'PATCHES/patch.000','PATCHES/patch.001','patch.bat'],check=True)

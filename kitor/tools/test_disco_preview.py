"""A CSV edit must never silently ship yesterday's preview delta."""
import hashlib,json,tempfile,unittest
from pathlib import Path
import build_patch_packages as build
class DiscoPreviewTests(unittest.TestCase):
 def test_stale_csv_fails_before_payload_access(self):
  with tempfile.TemporaryDirectory() as d:
   root=Path(d);(root/'patch.preview.json').write_text(json.dumps({'scummvm_id':'disco-elysium'}));(root/'translations.csv').write_text('changed');(root/'package-build.json').write_text(json.dumps({'csv_sha256':hashlib.sha256(b'old').hexdigest()}))
   with self.assertRaisesRegex(build.InvalidPackage,'Disco preview is stale'):build.validate(root/'patch.preview.json','osx')
 def test_changed_compiler_fails_before_payload_access(self):
  with tempfile.TemporaryDirectory() as d:
   root=Path(d);(root/'patch.preview.json').write_text(json.dumps({'scummvm_id':'disco-elysium'}));(root/'translations.csv').write_bytes(b'csv');(root/'build_package.py').write_bytes(b'changed')
   (root/'package-build.json').write_text(json.dumps({'csv_sha256':hashlib.sha256(b'csv').hexdigest(),'compiler_sha256':hashlib.sha256(b'old').hexdigest()}))
   with self.assertRaisesRegex(build.InvalidPackage,'Disco preview is stale'):build.validate(root/'patch.preview.json','osx')

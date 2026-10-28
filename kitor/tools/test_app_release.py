import copy
import unittest
from app_release import PLATFORMS, REPO, filename, manifest_for, version_tuple


class AppReleaseTests(unittest.TestCase):
    def setUp(self):
        self.release = {"tag_name": "app-v0.2.0", "prerelease": False, "assets": [], "body": "Release notes"}
        self.signatures = {}
        for platform in PLATFORMS:
            name = filename("0.2.0", platform)
            self.release["assets"].append({"name": name, "size": 42, "browser_download_url": f"https://github.com/{REPO}/releases/download/app-v0.2.0/{name}"})
            self.signatures[name + ".sig"] = "signed-test-fixture"

    def test_manifest_maps_all_platforms_to_versioned_assets(self):
        manifest = manifest_for(self.release, self.signatures)
        self.assertEqual(manifest["version"], "0.2.0")
        self.assertEqual(manifest["notes"], "Release notes")
        self.assertEqual(set(manifest["platforms"]), set(PLATFORMS))
        self.assertNotIn("pub_date", manifest)
        self.release["published_at"] = "2026-10-28T12:00:00Z"
        self.assertEqual(manifest_for(self.release, self.signatures)["pub_date"], self.release["published_at"])

    def test_missing_platform_or_signature_blocks_feed(self):
        incomplete = copy.deepcopy(self.release)
        incomplete["assets"].pop()
        with self.assertRaises(KeyError): manifest_for(incomplete, self.signatures)
        with self.assertRaises(KeyError): manifest_for(self.release, {})
        signatures = dict.fromkeys(self.signatures, "")
        with self.assertRaises(ValueError): manifest_for(self.release, signatures)

    def test_patch_and_prerelease_tags_are_rejected(self):
        for tag in ["patch-sq2-v1", "app-v0.2.0-beta", "app-v01.2.0", "0.2.0"]:
            self.release["tag_name"] = tag
            with self.assertRaises(ValueError): manifest_for(self.release, self.signatures)
        self.release["tag_name"] = "app-v0.2.0"
        self.release["prerelease"] = True
        with self.assertRaises(ValueError): manifest_for(self.release, self.signatures)

    def test_foreign_download_urls_and_duplicate_assets_are_rejected(self):
        original = copy.deepcopy(self.release)
        self.release["assets"][0]["browser_download_url"] = "https://evil.example/app.exe"
        with self.assertRaises(ValueError): manifest_for(self.release, self.signatures)
        original["assets"].append(original["assets"][0])
        with self.assertRaises(ValueError): manifest_for(original, self.signatures)

    def test_feed_version_comparison_is_numeric(self):
        self.assertGreater(version_tuple("1.10.0"), version_tuple("1.9.0"))
        for bad in ["1.2", "1.2.3-rc1", "01.2.3", "v1.2.3"]:
            with self.assertRaises(ValueError): version_tuple(bad)

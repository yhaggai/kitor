"""Regression checks for the translation-only CI gate."""

import csv
import io
import json
import unittest
from unittest.mock import patch

import check_translation_pr as checker


class TranslationScopeTests(unittest.TestCase):
    def test_reference_catalog_preserves_source_and_identity(self):
        old = [{"path": ["line", 0], "en": "Hello {name}", "he": "שלום {name}"}]
        new = [{**old[0], "he": "היי {name}"}]
        self.assertTrue(checker.validate_reference_catalog(json.dumps(old), json.dumps(new)))
        new[0]["path"] = ["line", 1]
        self.assertFalse(checker.validate_reference_catalog(json.dumps(old), json.dumps(new)))
        new[0] = {**old[0], "he": "היי"}
        with self.assertRaises(ValueError):
            checker.validate_reference_catalog(json.dumps(old), json.dumps(new))

    def test_editorial_csv_only_allows_matching_hebrew_edits(self):
        header = ["stable_id", "path", "category", "scene", "speaker", "en", "he_unpointed", "he_pointed"]
        def encode(row):
            stream = io.StringIO(newline="")
            csv.writer(stream).writerows([header, row])
            return stream.getvalue().encode("utf-8-sig")
        old = ["id", "[]", "content", "scene", "speaker", "Hello", "שלום", "שָׁלוֹם"]
        new = [*old[:6], "היי", "הַיי"]
        self.assertTrue(checker.validate_editorial_csv(encode(old), encode(new)))
        new[5] = "Changed English"
        self.assertFalse(checker.validate_editorial_csv(encode(old), encode(new)))
        new[5] = old[5]
        new[7] = "שלום"
        self.assertFalse(checker.validate_editorial_csv(encode(old), encode(new)))

    def test_mixed_source_change_requires_full_ci(self):
        with patch.object(checker, "git", return_value=b"M\0kitor/src/App.tsx\0"):
            self.assertFalse(checker.translation_only("base"))

    def test_executable_translation_requires_full_ci(self):
        with patch.object(checker, "git", side_effect=[
            b"M\0patches-workspace/game/he/catalog.csv\0", b"100755 blob abc\tfile"
        ]):
            self.assertFalse(checker.translation_only("base"))

    def test_unknown_content_requires_full_ci(self):
        self.assertIsNone(checker.validator("payload/run.py"))
        self.assertIsNone(checker.validator("patch.json"))


if __name__ == "__main__":
    unittest.main()

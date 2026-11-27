"""PR routing coverage independent of GitHub runner execution."""

import unittest
from unittest.mock import patch

import check_pr_scope as scope


class PrScopeTests(unittest.TestCase):
    def classify(self, paths, translation_only=False):
        with patch.object(scope, 'git', return_value=('\0'.join(paths) + '\0').encode()), \
             patch.object(scope, 'translation_only', return_value=translation_only):
            return scope.classify('base')

    def test_translation_only_is_shared_with_both_build_routes(self):
        result = self.classify(['patches-workspace/game/he/catalog.csv'], True)
        self.assertEqual(result, {'translation_only': True, 'installers': True, 'packages': True,
                                  'workflow_or_docs_only': False})

    def test_frontend_change_only_routes_installers(self):
        self.assertEqual(self.classify(['kitor/src/App.tsx']),
                         {'translation_only': False, 'installers': True, 'packages': False,
                          'workflow_or_docs_only': False})

    def test_backend_and_deleted_patch_files_route_both_builds(self):
        for path in ['kitor/src-tauri/src/lib.rs', 'patches-workspace/game/he/deleted.bin']:
            with self.subTest(path=path):
                result = self.classify([path])
                self.assertTrue(result['installers'])
                self.assertTrue(result['packages'])

    def test_unrelated_docs_do_not_route_builds(self):
        result = self.classify(['README.md'])
        self.assertFalse(result['installers'])
        self.assertFalse(result['packages'])

    def test_workflow_and_markdown_changes_can_skip_codeql(self):
        result = self.classify(['docs/README.md', '.github/workflows/check.yaml'])
        self.assertTrue(result['workflow_or_docs_only'])

    def test_empty_or_mixed_changes_cannot_skip_codeql(self):
        self.assertFalse(self.classify([])['workflow_or_docs_only'])
        result = self.classify(['.github/workflows/check.yml', 'kitor/src/App.tsx'])
        self.assertFalse(result['workflow_or_docs_only'])

    def test_mixed_changes_cannot_skip_full_checks(self):
        result = self.classify(['patches-workspace/game/he/catalog.csv', 'kitor/src/App.tsx'])
        self.assertFalse(result['translation_only'])
        self.assertTrue(result['installers'])
        self.assertTrue(result['packages'])

    def test_scope_infrastructure_routes_both_builds(self):
        for path in ['.github/workflows/classify-translation-pr.yml',
                     'kitor/tools/check_translation_pr.py', 'kitor/tools/check_pr_scope.py']:
            with self.subTest(path=path):
                result = self.classify([path])
                self.assertTrue(result['installers'])
                self.assertTrue(result['packages'])

    def test_globs_respect_directory_boundaries(self):
        self.assertTrue(scope.paths_match(['kitor/tsconfig.app.json'], ['kitor/tsconfig*.json']))
        self.assertFalse(scope.paths_match(['kitor/tsconfig/nested.json'], ['kitor/tsconfig*.json']))
        self.assertTrue(scope.paths_match(['kitor/src/deep/file.ts'], ['kitor/src/**']))


if __name__ == '__main__':
    unittest.main()

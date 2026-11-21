"""Route PR checks using the same shallow comparison as translation validation."""

import re
import sys

from check_translation_pr import git, translation_only

INSTALLERS_PATHS = (
    '.github/workflows/classify-translation-pr.yml',
    'kitor/tools/check_translation_pr.py',
    'kitor/src/**',
    'kitor/src-tauri/**',
    'kitor/native-steam/**',
    'kitor/public/**',
    'packages/ui/**',
    'kitor/patches/**',
    'patches-workspace/**',
    'kitor/tests/**',
    'kitor/tools/app_release.py',
    'kitor/tools/test_app_release.py',
    'kitor/package.json',
    'kitor/index.html',
    'kitor/postcss.config.js',
    'kitor/tailwind.config.js',
    'kitor/tsconfig*.json',
    'kitor/vite.config.ts',
    'package.json',
    'bun.lock',
    '.github/workflows/build-installers.yml',
    '.github/workflows/release-app.yml',
    'kitor/tools/check_pr_scope.py',
    'kitor/tools/test_check_pr_scope.py',
)

PACKAGES_PATHS = (
    '.github/workflows/classify-translation-pr.yml',
    'kitor/tools/check_translation_pr.py',
    'patches-workspace/**',
    'kitor/tools/**',
    'kitor/src-tauri/**',
    'kitor/patches/**',
    '.github/workflows/validate-patch-packages.yml',
    '.github/workflows/deploy-patches.yml',
    'kitor/tools/check_pr_scope.py',
    'kitor/tools/test_check_pr_scope.py',
)


def paths_match(paths, patterns):
    # These filters use only * (one path component) and ** (any depth).
    expressions = [re.escape(pattern).replace(r"\*\*", ".*").replace(r"\*", "[^/]*")
                   for pattern in patterns]
    return any(re.fullmatch(expression, path) for path in paths for expression in expressions)


def classify(base):
    paths = git("diff", "--name-only", "--no-renames", "-z", base, "HEAD").decode().split("\0")
    return {
        "translation_only": translation_only(base),
        "installers": paths_match(paths, INSTALLERS_PATHS),
        "packages": paths_match(paths, PACKAGES_PATHS),
    }


if __name__ == "__main__":
    for name, enabled in classify(sys.argv[1]).items():
        print(f"{name}={str(enabled).lower()}")

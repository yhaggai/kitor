---
name: kitor-github-actions
description: Add, edit, or review Kitor GitHub Actions workflows and composite actions. Check action pins before committing or opening a pull request.
---

# GitHub Actions in Kitor

- Pin every external action and reusable workflow to its full 40-character commit SHA. Add the corresponding release tag as a comment when known. Local `./` references use repository-relative paths.
- Search the repository for an existing pin for the same dependency and release. For a new dependency or version, resolve the release tag to its commit in the upstream repository. Peel annotated tags to their commit and verify the SHA comes from the upstream repository.
- Review every added or modified `uses:` reference in the diff, including job-level reusable workflows and composite action steps. Run zizmor on the changed workflows when available and verify its GitHub code-scanning result after pushing.
- If zizmor flags an unpinned reference, fix the reference and rerun the check.

---
name: separate-test-files
description: Keep tests in dedicated files when adding, editing, moving, or reviewing tests in Kitor.
---

# Separate test files

Put test cases, test-only helpers, and fixtures in dedicated test files. Follow the package's existing test naming and discovery conventions.

For Rust unit tests that need private access, keep a `#[cfg(test)]` module declaration in the source and define its contents in a separate file. Use Cargo's `tests/` directory for integration tests. When moving tests, preserve their module relationship and check any relative fixture paths.

Verify that the normal test command discovers and runs the affected tests.

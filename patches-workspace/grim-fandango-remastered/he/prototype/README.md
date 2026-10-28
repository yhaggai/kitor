# Grim Fandango Hebrew build prototypes

Build tools live inside the Hebrew package, grouped by platform:

- [Windows](windows/README.md): native-game resource builder, launch helpers,
  packaging tool, requirements, and tests.
- [macOS](macos/README.md): the earlier ScummVM-based experiment and launcher.

Each prototype keeps its generated `runtime/` data beside its scripts and out of
Git. The package manifest, payload, licenses, provenance, and validation remain
in the parent `he/` directory.

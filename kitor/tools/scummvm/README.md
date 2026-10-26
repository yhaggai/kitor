# Toonstruck Hebrew runtime fix

ScummVM v2026.4.0's `engines/toon/font.cpp` takes `c_str()` from a temporary
`Common::String` returned by `convertBiDiString`, then draws after the temporary
is destroyed. Both single-line and multiline Hebrew rendering are affected.
The patch retains an owning string through each drawing loop.

## Local macOS build

Install the build prerequisites (`brew install sdl2 fribidi pkgconf`) and run:

```sh
bash kitor/tools/scummvm/build-toon-macos.sh /tmp/kitor-toon-build
```

The script downloads and verifies the pinned v2026.4.0 source, applies the patch,
and builds only the Toon engine. It does not replace the shared runtime.
Copy the resulting `scummvm` executable and `dists/engine-data/toon.dat` into
`<Kitor app data>/scummvm-toon-hebrew/`. On this Mac the app data directory is
`~/Library/Application Support/haggai/`.

Kitor selects this runtime only for Toonstruck and only when both files exist,
and passes that directory as `--extrapath` for the engine data. Other games keep
using the standard runtime. Removing/renaming the sidecar directory restores
standard Toonstruck runtime selection. No original game files are modified.

This is a local arm64 build linked to Homebrew SDL2 and FriBiDi, not a portable
release artifact. Windows builds and packaging for other Macs remain pending.
Shared-runtime downloads do not install this fix on other computers.

## Validation

- Built on macOS arm64 with FriBiDi enabled.
- Launched the owned GOG edition as DOS/Hebrew with isolated test configuration.
- User confirmed the previously unstable subtitles work in the patched build.
- Rust regression test covers optional runtime selection, missing engine data,
  other-game isolation, and the engine-data argument.

Source: https://github.com/scummvm/scummvm/tree/v2026.4.0
Source archive SHA-256:
`e9615bad3794121be740aef7270fe3bec75e9b372fbaadaf8a07bb8fe2aa016a`

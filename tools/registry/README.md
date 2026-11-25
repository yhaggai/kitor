# Registry schema and shared vocabulary

`packages/registry/src/index.ts` defines the registry vocabulary and its strong
TypeScript types. App and shared UI consumers import `Tag`, `Store`, `Platform`,
`PatchLanguage`, and source constants from `@kitor/registry`. This entry point has
no Zod runtime dependency, keeping UI imports lightweight.

`packages/registry/src/schema.ts` builds Zod enums from that vocabulary and exports
registry schemas, inferred structural types, and `validateRegistry` through
`@kitor/registry/schema`. `tools/registry/schema.ts` re-exports it for existing
validation callers. Identifiers, versions, paths, arguments, and descriptions
remain strings. `CatalogSourceType` also includes `store` for store-only API
entries; it is not an installable registry source. `local-preview` represents the
existing development-only local package source.

From the repository root:

```sh
bun run registry:validate
bun run registry:test
bun run registry:typecheck
bun run ui:check
```

After changing the vocabulary or schema:

```sh
bun run registry:schema
```

Commit both generated outputs: `kitor/patches/registry.schema.json` for editor
validation and `kitor/src-tauri/src/services/registry_vocabulary.rs` for native
registry models, launch/detection capabilities, and patch platform selection. CI
checks these files for drift, tests consumer behavior, and typechecks the app/UI.
Do not edit generated files directly. Category labels and icons use exhaustive
records so new tags require corresponding UI entries.

VS Code associates the registry JSON with its generated schema through workspace
settings. Objects reject unknown fields to catch spelling mistakes. Optional and
nullable fields preserve existing Rust registry conventions. Cross-record checks
also reject duplicate game identities and incompatible launch capabilities; these
checks are not represented in JSON Schema. Runtime path safety, downloads, game
compatibility and gameplay still require their existing checks. Persisted database
strings are distinct from the typed registry vocabulary.

The schema covers the registry, not lifecycle package manifests or
`bundled-assets.json`. Zod exports the structural definition to JSON Schema:
https://zod.dev/json-schema.

## Native launch capabilities

Native profiles use `commands` keyed by the `Platform` enum. There is no
per-game native-launch selector. `NativeCommand` and `LaunchProfile` are exported
from `@kitor/registry/schema`; launch enums come from `@kitor/registry` and are
also generated as Rust enums.

```json
{
  "commands": {
    "windows": {
      "program": "Game.exe",
      "required_files": ["data.bin"],
      "env": { "SteamAppId": "12345" }
    },
    "osx": {
      "root": "app_bundle",
      "bundle_name": "Game.app",
      "method": "app_bundle",
      "program": "Contents/MacOS/Game",
      "working_directory": "bundle_resources",
      "executable_files": ["Contents/MacOS/Game"]
    }
  }
}
```

- `method`: `executable` (default) runs the relative `program`; `app_bundle`
  opens the resolved bundle through `/usr/bin/open`, passing configured game
  arguments after `--args`. The declared program must exist in both modes.
- `root`: `game_directory` (default) uses the detected installation path;
  `app_bundle` accepts a bundle, a path inside its Contents, or a parent containing
  `bundle_name`. Bundle names are single `.app` directory names. Bundle roots
  require macOS. Case-insensitive lookup handles Windows-origin asset names.
- `working_directory`: `root` (default), `bundle_parent`, or `bundle_resources`.
  The latter two require an app-bundle root.
- `args` and `env` configure executable invocation. Bundle mode supports arguments
  but rejects environment settings, since LaunchServices does not reliably pass
  the opener process environment to the app; use executable mode when needed.
- `required_files`, `required_directories`, and `signatures` validate the edition
  before preparing a launch. A signature has a relative `path` and a `prefix`
  array of 1–4096 byte values. Monkey Island and DOTT use this for KAPL headers.
- `executable_files` restores only the owner-execute bit on Unix. All paths and
  prerequisites are checked first. Escaping paths, symlinks, and ambiguous
  case-insensitive aliases are rejected.
- `preparation` selects a custom native step only when configuration cannot
  express it. Currently only `grim_hebrew_preferences` remains: its binary
  settings format needs code. It is set only on Grim's patched Windows command.
- Profile-level `validate_on_import` opts into the same launch validation and
  permission repair before publishing a managed Steam download. Owlboy uses it
  to preserve its previous import checks; other downloads retain their flow.

The previous `native`, `native_game_id`, and `prepare_hebrew_preferences` fields
have been replaced in the checked-in registry. ScummVM preparation and
format-specific installation remains a separate concern. Native execution
uses `services/native_launch.rs`; standalone patch tools use that same path.

## Generic detection

`DetectionMode` has `patterns` and `layouts`; it contains no game identities.
Patterns retain the existing case-insensitive `unique_files` discovery. Layouts
are ordered alternatives, each requiring all of its evidence. Once layouts are
configured, a failed rule cannot fall back to a weak filename match.

```json
{
  "mode": "layouts",
  "layouts": [{
    "root": ".",
    "required_files": ["Game.exe", "Game_Data/app.info"],
    "file_checks": [{
      "kind": "text",
      "path": "Game_Data/app.info",
      "equals": "Studio\nGame",
      "normalization": "trim_crlf"
    }]
  }]
}
```

- `root` and `data` are relative paths; `.` or an empty value denotes the root.
  Required files/directories resolve case-insensitively and reject symlinks and
  ambiguous names. `data` chooses the detected installation/data directory.
- `ancestor` selects zero to three parent levels before resolving `root`.
  `selected_name` can restrict that rule to input folders such as `Resources`.
  Parent traversal is explicit and bounded; `..` in registry paths is rejected.
- `path_style` is `canonical` by default, or `preserve` when existing library paths
  must retain their spelling. Space Quest, Toonstruck, and Torin retain their
  previous path behavior.
- `file_checks` is a strongly typed discriminated union: `text` compares exact
  UTF-8 text or removes CR characters and trims outer whitespace; `prefix` checks
  bytes and an optional minimum file size; `digest` checks an accepted SHA-256 or
  MD5 value over the whole file or a bounded prefix, optionally with an exact
  file size; `size` requires a minimum file size. Hashes are streamed.
- `edition` labels `classic` or `remastered` rules for format-specific package
  tools. Those entry points select registry rules; they contain no matching code.
- Existing detection priority and additional scan subdirectories remain registry
  data. macOS GOG wrapper paths retain their existing discovery support.

Disco Elysium uses identifying `app.info` text plus complete platform assets.
Owlboy uses complete FNA layouts; Monkey Island and Tentacle use KAPL headers;
Space Quest retains exact known SHA-256 values; Torin retains its sized, 5000-byte
MD5 checks. The Space Quest JSON under `game_service/fixtures` is an independent
owned-test snapshot, not another runtime catalog.

Runtime implementation: `services/game_detection.rs`. Authoring types:
`DetectionLayout` and `DetectionFileCheck` from `@kitor/registry/schema`; mode,
digest, normalization, path-style, and edition enums from `@kitor/registry`.

# Rust security patches

Cargo uses local copies of these crates because the current Tauri and SQLx releases constrain the affected dependencies. The copies retain the upstream crate versions and licenses.

| Source package | Local change |
| --- | --- |
| `glib 0.18.5` | Backport the [upstream `VariantStrIter` fix](https://github.com/gtk-rs/gtk-rs-core/pull/1343) for [RUSTSEC-2024-0429](https://rustsec.org/advisories/RUSTSEC-2024-0429.html). |
| `sqlx-mysql 0.8.6` | Use RSA 0.10 and its OAEP API to resolve [RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071.html). MySQL authentication keeps OAEP with SHA-1 for protocol compatibility. |
| `sqlx-sqlite 0.8.6` | Allow `libsqlite3-sys 0.39`, which bundles SQLite 3.54 and fixes [RUSTSEC-2026-0459](https://rustsec.org/advisories/RUSTSEC-2026-0459.html). |
| `glib-macros 0.18.5` and `gtk3-macros 0.18.2` | Use `syn::Error` for macro diagnostics in place of [the unmaintained `proc-macro-error`](https://rustsec.org/advisories/RUSTSEC-2024-0370.html). |
| `tauri-plugin-opener 2.7.3` | Use zbus 6 for the D-Bus file opener. The release incorporates the formerly standalone [wire types](https://rustsec.org/advisories/RUSTSEC-2026-0370.html), [name types](https://rustsec.org/advisories/RUSTSEC-2026-0371.html), and [derives](https://rustsec.org/advisories/RUSTSEC-2026-0372.html). |
| `keyring 3.6.3` | Use the local copy under `native-steam/vendor` in both Cargo projects. Its `sync-crypto-rust` feature selects the existing encrypted synchronous Secret Service backend without resolving the unused async backend and its zbus 4 dependencies. |

The directories come from the crates.io release archives, verified against the checksums in the previous lockfile. The `[patch.crates-io]` section in the parent manifest selects the local copies. Remove each override when the supported upstream dependency chain includes its fix.

The [OSV configuration](../osv-scanner.toml) records the `glib` backport for the adjacent lockfile because the patched copy retains version 0.18.5. Remove the advisory entry when removing the `glib` override.

# Kitor native Steam content transport

This crate is linked into Kitor's Rust backend. It does not invoke Steam,
DepotDownloader, SteamKit, .NET, or another downloader executable. The small
`kitor-steam` binary is a developer harness for exercising the same library.

Connect Steam in Kitor's settings with QR approval or username/password and Steam Guard.
The session is stored in the OS credential store under
`click.kitor.steam` / `native-content-session`. The same session supplies owned
games through authenticated `Player.GetOwnedGames` and authorizes downloads.
No separate Web API key is required. Library refresh runs at startup, after
sign-in, and on request in settings. Logout clears the displayed ownership list
and removes the native session. Legacy saved Web API credentials are left intact
but are no longer used by the library UI.

The transport accepts only app/depot pairs selected by Kitor's curated product
catalog when called from the desktop app. Steam validates the session, depot key,
and manifest request. Family-shared games are excluded from personal ownership;
a successful public metadata request is never treated as download permission.

Files are downloaded into a temporary directory, with bounded parallel requests,
AES decryption, chunk and whole-file SHA-1 verification against the manifest, and
path validation. The directory is published only after all files verify. Existing
destinations are rejected. Importers then perform their existing edition checks,
register games, and hand off to the existing patch installer. Kitor-owned verified
temporary content is moved into installation staging on the same filesystem,
avoiding a second copy and checksum pass. Cross-filesystem imports retain the
verified-copy fallback. Existing Steam-client content and reusable caches are
never consumed. Patch verification and backup checks remain unchanged.

The current transport supports public-branch manifests and the compression
formats ZIP, Valve LZMA, and Valve Zstandard. Symlink entries are rejected. It does
not implement interrupted-download resume, private beta branches, Steam library
registration, or Steam Cloud saves. It has been exercised against Owlboy's macOS
depot; other curated titles still require live validation.

## Development

```sh
cargo build --manifest-path kitor/native-steam/Cargo.toml
kitor/native-steam/target/debug/kitor-steam connect
kitor/native-steam/target/debug/kitor-steam download 115800 115805 /absolute/new/folder
```

This native session requires its own QR approval or password sign-in, even if
another tool has already authenticated. Passwords are RSA-encrypted for Steam,
are not persisted, and are cleared from the form after submission. Steam Guard
email codes, authenticator codes, and approval links are handled in the app. The crate does not extract credentials from Steam or
DepotDownloader. On macOS, rebuilding an unsigned development binary may require
new Keychain access approval.

Protocol field numbers, enum values, and content formats were researched from
SteamRE's publicly available SteamKit protocol declarations and content-format
implementations. No DepotDownloader or SteamKit source files are vendored or
linked. This is Kitor-authored Rust code, not a line-for-line source translation.
Dependencies for networking, cryptography, compression, QR rendering, and OS
credential storage are declared in Cargo.toml and pinned in Cargo.lock.

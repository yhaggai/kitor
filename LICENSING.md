# Licensing scope

The [MIT license](LICENSE) applies to original source code, build scripts, and
documentation authored by Haggai Yaniv for Kitor, including the desktop app,
website, and original patch tooling, unless a file carries another license.
It grants only rights held by that author.

It does not relicense third-party dependencies, upstream code, game content,
community translations, patch payloads, fonts, logos, cover images, or other
artwork. This includes third-party material under `patches-workspace/`,
`patches-metadata/`, `assets/`, and application/website asset directories.
Original code in those directories remains covered as described above;
the directory name alone does not establish ownership of its contents.
Preserve upstream licenses and notices. Credits and source URLs establish
provenance, not permission to redistribute or relicense.

For patch-specific origins and restrictions, consult each game's
`provenance.json`, `README.md`, and `licenses/` directory, plus
`kitor/patches/registry.json`. Missing license information is not an MIT grant.
The project has not yet completed an audit of all third-party assets.

Before submitting an installer for third-party signing, inventory its actual
contents and document each component's license. Review downloaded packages
separately and disclose their relationship to the app to the signing provider.

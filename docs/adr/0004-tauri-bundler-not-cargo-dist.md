---
status: accepted
---

# Distribution uses the Tauri bundler, and Homebrew waits for a Developer ID

The sibling project `plz` ships through cargo-dist, and a reader who knows it
will ask why Demysto does not. cargo-dist's model is a bare binary dropped into
`~/.local/bin`; Demysto is an `.app` bundle with `Info.plist`, `LSUIElement`, an
icon, a tray and a webview, delivered as `.dmg`, `.msi`/NSIS and `.AppImage`.
cargo-dist has no representation for any of that, and the Tauri bundler does.

macOS ships as a **universal** artifact via `tauri build --target
universal-apple-darwin`, which builds both architectures and runs `lipo` itself.
This is also the concrete thing cargo-dist could not do — its `targets` list
accepts only real rustc triples, which is why `plz` ships two archives and
explains `uname -m` in its README. Doubling build time and binary size (~8 → ~16 MB)
is worth deleting that paragraph from a tool whose stated purpose is fewer steps.

## Homebrew

`brew install --cask demysto` from a personal tap is possible and wanted, but it
is gated on code signing rather than shipped in v1. From 2026-09-01 Homebrew
disables casks that fail Gatekeeper in the main repository and has removed the
`--no-quarantine` escape hatch; an unsigned cask in a personal tap still installs,
but leaves the user to run `xattr -rd com.apple.quarantine` by hand — a worse
first run than downloading the `.dmg`. So the Developer ID, notarization, the
Homebrew cask and the `winget` manifest are one milestone, not four.

## Consequences

In-app updates do not wait for any of this: `tauri-plugin-updater` reads a
`latest.json` from GitHub Releases and is signed with its own keypair, unrelated
to Apple or Microsoft certificates. That keypair is generated before the first
release, because changing it afterwards strands every installed copy. The macOS
bundle is signed too, with a certificate of the project's own — what that is for,
and what it is not, is ADR-0015.

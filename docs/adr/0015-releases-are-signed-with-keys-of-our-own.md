# ADR-0015: Releases are signed with keys of our own, there being no Developer ID

Status: accepted

## Context

Two signatures are in play, and taking them for one thing is the mistake this
record exists to prevent.

The first is the updater's. `tauri-plugin-updater` verifies every artifact it
downloads against a minisign public key compiled into the application, whose
private half signs the release. That keypair is Demysto's own and answers to
nobody: it is what makes an update safe to take, and it would be exactly as
trustworthy if Apple and Microsoft did not exist. It has to be generated before
the first release, because changing it afterwards strands every installed copy
with no path forward.

The second is the platform's, and it is the one nobody is going to sell us. A
Developer ID is $99 a year and a Microsoft certificate more; neither is being
bought, now or soon. What that costs is a warning at the first launch on both
platforms — SmartScreen once, and on macOS the download's quarantine, cleared by
opening the application from its context menu the first time.

On macOS it costs one thing more, and that one is not a warning. An ad-hoc
signature's designated requirement is `cdhash H"…"` — the hash of that exact
build — and it is the requirement, not the path or the bundle identifier, that
macOS stores when the user grants Accessibility. Every update changes the hash.
The grant then matches nothing, and the permission does not fail loudly: the
Capture goes on returning the clipboard, and the user meets the disguised
failure ADR-0011 is about. An update would break the tool and say nothing.

## Decision

The updater keypair is generated before the first release and its private half
held as a repository secret, per the spec.

macOS builds are signed with a **self-signed** code-signing certificate, held as
a repository secret alongside it. It buys nothing from Gatekeeper — the
quarantine warning is identical either way — and it buys the only thing that
matters here: the designated requirement becomes

```
identifier "app.demysto" and certificate root = H"…"
```

which names the identifier and the certificate rather than this one build, and
goes on being satisfied by every later build the same certificate signs. The
Accessibility grant survives an update. This was measured on the macOS stand
rather than reasoned about: an installed copy was granted the permission with
the requirement macOS itself would have stored, updated itself to the next
version, and still had it afterwards — while the same run with an ad-hoc
signature lost it.

Windows is left unsigned. There is no equivalent to lose there: SmartScreen
warns once about the installer and nothing is revoked afterwards.

## Consequences

The certificate is now something to keep. Replacing it moves the requirement,
which costs every installed copy its permission exactly once — so it is issued
for ten years and backed up with the updater key, and neither is generated
casually.

It has to be trusted where the build happens, because `codesign` refuses an
identity it does not trust. That is a step in the release workflow and a
property of the build machine alone: a user's machine evaluates the requirement
by comparing a hash, and asks nobody whether the certificate deserves it.

The requirement is weaker than a Developer ID's, and weaker in a specific way:
it is satisfied by anything signed with this certificate, so a leak of the
private half is a leak of Demysto's identity to macOS. It is stronger than what
is being replaced, where the requirement was satisfied by whatever build the
user happened to grant.

Notarization, the Homebrew cask and a `winget` manifest still wait on a
Developer ID, as ADR-0004 says. Nothing here brings them closer; it removes the
one consequence of not having one that was not merely cosmetic.

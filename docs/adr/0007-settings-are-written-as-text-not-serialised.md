---
status: accepted
---

# The settings window edits the file's text, not the settings it parsed to

Ticket 08 gives the settings file a window to be edited from. That window could
have written the file by serialising the settings back out — one small step from
the `serde` types that already read them. It does not. It parses the file with
`toml_edit`, sets the fields it owns, and renders the same document back.

Serialising would flatten the file. What a fresh installation gets is a page of
prose explaining what each field means and which presets exist, followed by a
commented-out example; a user may have added comments of their own, ordered
their Providers to taste, or be running a newer Demysto that wrote a field this
one does not understand. All of that is data to the person who owns the file and
none of it survives a round trip through a `struct`. ADR-0005 says the config
directory belongs to the user. The window is a guest in it.

Three consequences follow from the same principle, and are part of this decision:

- A save is **validated by being read back** before it is written: the rendered
  document is parsed and resolved, and only a file Demysto could act on reaches
  the disk. A window that had written a file it could no longer open would leave
  the user repairing by hand the one file this ticket exists to spare them.
  Configuring no Provider at all is the single exception, because emptying the
  file is what starting over passes through.
- The file is **written beside itself and renamed over**, owner-only, rather
  than truncated and filled in. It holds a key, and a crash between the
  truncation and the last byte is a user whose credentials are simply gone.
  ADR-0002's mode `0600` is carried by the new file rather than inherited.
- A Provider carries **the name it had** as well as the name it is to have, so
  that renaming one finds the table the file already holds for it — and with it
  the key the window was never shown, and any comment written above it.

## Consequences

Demysto depends on `toml_edit` as well as `toml`, and two parsers can in
principle disagree about one file. The window reports that as a file it could
not edit, and writes nothing.

The settings are no longer read once. They are read at startup and again on
every save, which is why the environment is snapshotted at startup rather than
consulted: a key that changed between two reads is a key nobody can reason
about, and the spec's *Core modules* asks for exactly that guarantee.

---
status: accepted
---

# An Action file is written by serialising it, unlike the settings file

ADR-0007 has the settings window edit the *text* of `settings.toml` with
`toml_edit`, so that the preamble, the user's own comments and any field a
newer Demysto wrote all survive a save. The window that writes an Action does
not do that. It serialises the Action and replaces the file.

The two files are not the same kind of object. `settings.toml` is one file
holding everything, met on a fresh installation as a page of prose with a
commented-out example under it, and holding a key this application promises
never to show back. An Action file is one small document whose every field the
window owns and displays: a name, a prompt, a Model, a Hotkey, and the
Parameters. There is no preamble to preserve, no secret to avoid reading, and no
field the window is not already editing — so there is nothing for a round trip
through `toml_edit` to save that the window is not about to write anyway.

What that costs is a comment somebody wrote in their own Action file, which a
save from the window will drop. The trade is accepted because the alternative is
a second text-editing path to keep correct for a document with six fields, and
because an Action is the file people are expected to *send* rather than annotate
— ADR-0005 puts one Action in one file precisely so it can be handed to a
colleague.

Three properties are kept from ADR-0007's reasoning even so:

- The file is **written beside itself and renamed over**, owner-only, rather
  than truncated and filled in. That machinery is shared with the settings file
  rather than written twice.
- A save is **read back**: what the window shows afterwards is the catalogue as
  the directory then holds it, not what the window believed it sent.
- An **Override states only what the user changed**. A file that says nothing
  is deleted rather than written, which is what makes "reset to default" and
  "save a built-in unaltered" the same act — and what lets a later version
  improve a built-in's prompt for somebody who only ever bound a Model to it.

## Consequences

An Action file carries a `version`, like the settings file, so that one written
by a newer Demysto is reported and left alone rather than parsed by guesswork.
A file that cannot be read is reported *beside* the Actions rather than instead
of them: one bad file is no reason for the rest of somebody's Actions to
vanish.

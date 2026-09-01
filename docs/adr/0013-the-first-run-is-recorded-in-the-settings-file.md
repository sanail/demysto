# ADR-0013: The first run is recorded in the settings file, and is over when its window closes

Status: accepted

## Context

The flow a fresh installation is met by has to know two things: whether to
appear at all, and when it is over.

**Whether the settings file exists** is no answer to the first: `config::read`
creates it from the template on the first read, so by the time anything could
ask, it is there. **Whether anything is configured** is not an answer on its own
either — it would bring the flow back at every launch until a Provider was
saved, which is the tool nagging somebody who has already said no once.

## Decision

It is recorded: `welcomed = true`, written into the settings file through
`toml_edit` like every other save, so that the preamble and the user's own
comments survive it (ADR-0007). It is the one line in that file Demysto writes
for its own sake rather than the user's, and the preamble says so — including
that taking the line out walks them through the flow again, which is the only
way back to it and cheaper than a button in Settings nobody would look for.

A file that already configures a Provider answers *yes* as well, without the
record. Every settings file written before this field existed says nothing about
it, and the update that introduces the flow must not meet somebody who has been
using Demysto for months with a wizard. It is also what keeps a flow that could
not succeed from being offered: a Provider the flow configured under a name that
file already holds is a save the file refuses, and refuses at every attempt.

A file that cannot be read or parsed answers *yes* too, though nothing has been
through anything. The flow's whole business is writing a Provider into that
file, and Demysto will not write over one it could not read; Settings is where
such a file is reported and repaired.

The second question — when the flow is over — is answered by its window going
away, however it goes: the button at the end of it, Escape, or the close button.
All three are somebody who has been asked what the flow exists to ask, and the
alternative is a flow that returns at every launch until its last step is
reached. Everything it offers is in Settings afterwards.

## Consequences

The record is a field, and `File` denies unknown ones: a file this build has
written is a file an older Demysto refuses to parse, and refuses in Settings'
own words. That is the ordinary cost of adding any field, and it is what the
file's `version` is for; nothing here makes downgrading work, and nothing else
in this repository does either.

Two of the flow's steps write nothing here at all. The login items are the
operating system's list and are asked of it rather than remembered, so somebody
who removes Demysto from it there is not contradicted by a file of ours. The
Accessibility permission is macOS's, is asked at every Capture (ADR-0011), and
is not something the flow can confirm — so its step walks the user to the pane
and says that granting it afterwards works just as well.

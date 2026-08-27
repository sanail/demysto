---
status: accepted
---

# Built-in Actions live in code; disk holds only user Actions and overrides

Built-in Actions (explain, translate, summarize, describe image) are defined in
code rather than seeded as files into the config directory on first run. Seeding
them would be simpler to reason about and would make them discoverable, but the
config directory belongs to the user: once seeded, a new built-in Action added in
a later version could never reach anyone who installed an earlier one, because we
would have no right to write into a directory the user now owns.

On disk, `actions/` holds one file per user-authored Action, plus overrides for
built-in ones — an edited prompt, a bound Model, a personal Hotkey. "Reset to
default" deletes the override. One file per Action rather than a single list, so
an Action can be sent to another person as a file; sharing presets later then
costs nothing.

## Consequences

Built-in and user Actions are the same shape and run through the same path — the
distinction is only where the definition comes from. That is what makes "add your
own prompts" a property of the model rather than a feature bolted onto it.

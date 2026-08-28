---
status: accepted
---

# A key is verified with a real request to a Model, not with the Model list

User story 42 asks that a key be "tested against the Provider during setup, so
that I learn it is wrong immediately rather than at the first Run". The cheap
way to do that is the Model list: `GET /models` is already implemented, costs no
tokens, and is authenticated at most services.

At most. OpenRouter's Model list is public, and OpenRouter is one of the five
presets Demysto ships. A verification that fetched it would tell a user with a
mistyped key that their key was fine, and the story exists precisely so that it
does not.

So verification sends the request a Run sends: one word, to a Model the user
named, at the Provider as the window has it — a key just typed included, before
anything is saved. The status alone answers, and the stream is dropped at the
headers rather than read, so the Model is cut off after the few tokens it takes
a connection to close. Nothing vendor-specific is sent to make it small: a
`max_tokens` some Providers reject, or that a reasoning Model refuses, would
turn a working key into a failed verification.

This also matches the first-run order the spec fixes: "configure a Provider with
a key **and a Model**, verify it with a live request".

## Consequences

A verification costs a token or two, and cannot be offered before a Model has
been chosen — which is why the settings window puts "Fetch" beside "Verify key"
and leaves the second disabled until one is.

It tests three things at once and reports whichever failed in the Provider's own
words: that the endpoint answers, that the key is accepted, and that this
account may use this Model. The last is not what the story asked for and is the
more common setup failure of the two.

---
status: accepted
---

# A preset may declare a service that has no API key

ADR-0002 fixes where a Provider's key comes from, and every Provider was
required to have one. A Provider with nothing to send failed every Run with a
message telling the user to go and find a key that does not exist.

The services this is wrong for are the local ones. LM Studio and Ollama both
serve the OpenAI Chat Completions contract on this machine, and neither has
keys: their own documentation tells people to pass a placeholder string, because
the OpenAI client libraries insist on one. User story 31 is "configure any
OpenAI-compatible endpoint", and a local server is the commonest instance of it.

So a preset now carries whether the service has a key variable at all, and one
that does not resolves to "no key", which sends the request with no
`Authorization` header rather than with an invented value.

ADR-0002's resolution order is untouched: `api_key_env`, then the preset's
conventional variable, then `api_key` in the file. Only what happens when all
three come up empty depends on the preset. A key stated for a keyless service is
still used, so a local server put behind something that wants one still works.

## Consequences

Only a preset can declare a service keyless, and only where the user has said
nothing to the contrary. A Provider written out by hand — `base_url` and no
preset — still requires a key. So does one that names a variable in
`api_key_env`, even under a keyless preset, and even when that variable holds
nothing: naming it is the user saying this Provider is authenticated, and it
holds nothing routinely, because an application launched from the Finder or a
desktop entry never sees what a shell profile exported. Between them, no typo in
the file and no missing export can quietly turn authentication off for a
Provider the user meant to authenticate. Adding a keyless preset is therefore a
decision recorded here, not a field a user sets.

The failure mode this admits is a request sent without credentials, which is
refused. It is not the failure mode worth guarding against: that one is a key
sent somewhere nobody chose, and nothing here makes it more likely.

Overriding a keyless preset's `base_url` with a remote address sends
unauthenticated requests to that address. This is the user pointing Demysto at a
server and saying it needs no key, which is theirs to say.

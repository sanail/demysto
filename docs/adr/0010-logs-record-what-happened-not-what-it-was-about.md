# ADR-0010: Logs record what happened, not what it was about

Status: accepted

## Context

A bug report needs something to carry (user story 63), so Demysto writes local
log files and Settings opens the folder they are in.

Two promises already made pull against that. Nothing is sent anywhere except the
user's chosen Provider (user story 61), and history is kept only until the
application quits (user story 62) — which is why the Conversation store is in
memory and nowhere else. A log is the one thing in the application that survives
quitting and is written to disk beside the settings file that holds the key. A
log of prompts and answers would be precisely the on-disk history the
Conversation store was deliberately built not to keep, reintroduced through a
side door, and it would be the file a user is most likely to attach to a public
issue.

The usual answer — log everything at debug level and let the user turn it down —
puts the decision on somebody who will not read the file before attaching it.

## Decision

The log records the shape of what happened and never its content.

Written: the version and configuration directory at startup; per Turn, the Model
and base URL it went to and how many messages went with it; per outcome, how
many characters came back, or the failure in the sentence the user was already
shown. Reports that have no window to appear on — a Hotkey that could not be
claimed, an Action file that could not be read — go here too.

Never written: the Selection, any prompt, any answer, any follow-up question,
and any key. `config::Provider`, `config::Key`, `settings::KeyEdit` and
`model::Endpoint` all carry hand-written `Debug` implementations that print
`<not shown>` in place of a key, so that no future `{:?}` in a log line or a
panic message can undo this.

Files roll over at 512 KiB and three are kept, so the whole set can be attached
to a report without anybody thinking about it, and a machine left running for a
month is not storing a year of this.

## Consequences

A bug report can say which Model was asked, how large the Conversation was, how
much came back and what went wrong — enough to place most faults. It cannot say
what the user asked, which is exactly the class of fault the logs will not help
with: a prompt that produced a poor answer has to be described rather than
attached.

The log folder is safe to attach to a public issue without reading it first,
which is the property this is for. A test asserts it — a Run and a follow-up,
then the file checked for the Selection, the answer and the key.

Nothing here reports a failure of its own. There is nowhere for a logger to
report to, and a log that can take the application down is worse than no log.

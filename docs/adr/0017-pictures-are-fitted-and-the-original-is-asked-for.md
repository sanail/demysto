# ADR-0017: A picture is always fitted, and its original goes only when asked for

Status: accepted

## Context

A picture on the clipboard is whatever the screen it came from is. A full
capture of a modern display is several thousand pixels across and megabytes
once encoded, and because the message list is resent on every Turn, that weight
is paid again for every follow-up question in the Conversation.

Fitting it — a ceiling on the longest side — costs almost nothing in the common
case and everything in an uncommon one: eight-point text in a screenshot of a
contract, a dense table, a chart whose axis labels are the point of the
question. Which case a given picture is cannot be decided from the picture's
size, and it cannot be decided by the person either, because what they need to
know is not what the picture looks like but what a Model will still be able to
read in it.

Three placements for the decision were considered.

A **setting** in Settings, beside the Default Vision Model. Rejected because it
asks a question nobody can answer about themselves: "do you want pictures
shrunk" has no stable answer, only a per-picture one, and a setting turned on
last month is a wrong answer to today's screenshot half the time.

A **checkbox in the Palette**, remembering its last state. Better, in that it is
at least asked per picture — but still asked before there is any evidence, and
with two holes. An Action with its own Hotkey skips the Palette entirely, so the
checkbox has nowhere to live on the fastest path and would fall back to a
remembered state anyway. And the Palette is the one surface in Demysto whose
whole claim is that one keypress runs one thing; a control there is paid on
every picture to be useful on few.

The third is to make the decision **after** the answer, where the evidence is.

## Decision

Every Run sends a fitted picture: PNG, a ceiling of 1568 px on the longest side,
aspect ratio preserved, never enlarged. There is no setting governing this — not
in Settings, not in the Palette, not in an Action file.

The Conversation window offers to ask again at the original resolution, with the
weight that will be sent written on the button. Pressing it re-runs the Action on
the same Selection, and every Turn after it in that Conversation sends the
original as well.

The original is therefore held in memory alongside the fitted picture, which is
what makes the offer real rather than a promise to re-capture something the
clipboard no longer holds. That memory is bounded by releasing a Conversation's
pictures when the result window closes, and by a ceiling over what is held —
after which the Conversation is Sealed: readable, but not continuable.

## Consequences

The fast path acquires nothing. A picture is captured and asked about in exactly
the keystrokes text costs, and the question of resolution never comes up for the
majority of pictures where fitting costs nothing.

Where it does cost something, the recovery is one click, made by somebody
looking at the evidence — a disappointing answer — rather than guessing in
advance. The price of that click is on the click.

The cost is memory and its consequences, and they are not small: pictures held
for the offer, a release signal the shell must send the core when the result
window closes, a byte ceiling with its own eviction, and a state a Conversation
can be in that no text Conversation ever reaches. All of that exists to support
one button. Had the decision gone to a setting or a checkbox, none of it would
be needed, because nothing would have to be kept — and that, not the interface
question, is what a reversal of this decision would actually buy back.

The numbers — 1568 px, PNG, the byte ceiling — are constants recorded in
`docs/spec/0002`, not principles. Photographs fare worse under PNG than
screenshots do, and if they turn out to be the common case, the format is one
line.

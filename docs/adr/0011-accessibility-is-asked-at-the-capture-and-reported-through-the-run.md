# ADR-0011: Accessibility is asked at the Capture and reported through the Run

Status: accepted

## Context

macOS gates synthetic input behind the Accessibility permission, and withdraws
it whenever the binary's signature changes — which is every development build,
and every update until the application is signed and notarized.

Without it a Capture does not fail. enigo sends the copy chord, nothing receives
it, the clipboard is left holding whatever it held, and the Capture reports that
as the fallback: "From the clipboard", or "nothing is selected". So the failure
arrives disguised as one of the two states the tool is designed to have. At best
the user sees a Palette insisting nothing is selected while their Selection is
on screen in front of them; at worst Demysto explains the paragraph they copied
an hour ago, confidently and about the wrong thing.

The spec says Accessibility is re-checked at every Run. Two readings of that are
available: inside `Demysto::run`, or before each Capture. They name the same
moments — every Run begins with a Capture, the Palette's or the one an Action's
own Hotkey performs on its way past the Palette — and only one of them is a
place where the answer changes what happens next.

## Decision

The question is asked once per Capture, in `DesktopCapture::capture`, before the
clipboard is touched. A Capture that types into nothing — the clipboard-only one
Wayland gets, per ADR-0003 — asks nothing, because there is no permission it
could be missing.

A refusal is a `CaptureError::Permission` carrying the whole sentence, and the
sentence is written in `desktop`, the only module that knows such a permission
exists. It names the pane as well as the permission, so that it is followable by
somebody reading it in a notification, where there is no button.

A Run that finds no Selection asks why the Capture before it produced none, and
reports a refused permission as `RunError::Permission` rather than as "there is
nothing to run an Action on". This is what puts the sentence in front of
somebody who pressed an Action's own Hotkey and never saw a Palette at all
(user story 55). A clipboard that could not be read is deliberately not among
them: that leaves the user able to select something and press the Hotkey again,
which is exactly what the existing sentence tells them to do.

`AXIsProcessTrusted` is declared as four lines of FFI rather than taken from a
crate — one function with no arguments is the whole of Demysto's interest in the
Accessibility API. The variant that offers to ask for the permission is not used:
this runs on every Hotkey press, and a system dialog on every Hotkey press would
be its own kind of broken. Walking the user to the permission belongs to the
first-run flow.

Opening the pane is the shell's, not the core's: it is a URL the desktop
resolves, and `demysto-core` is not allowed to know that such a thing exists.
Both windows that can show the failure offer it as a button beside the sentence.

## Consequences

The permission is asked about once per Capture rather than once per session, so
a revocation is noticed at the next keypress instead of at the next launch —
which is the point, given how routinely macOS revokes it.

A permission withdrawn between the Palette opening and an Action being chosen in
it is not caught. The Run then works on the Selection already captured, which is
the right answer anyway: that text was read while the permission held.

Nothing here is tested against macOS. The suite exercises the whole surrounding
behaviour through the fake desktop at the seam the spec names — the refusal, the
clipboard left alone, the Run that reports it, and the clipboard-only session
that asks nothing — and whether `AXIsProcessTrusted` tells the truth is macOS's
business.

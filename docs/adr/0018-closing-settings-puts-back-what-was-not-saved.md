# ADR-0018: Closing Settings puts back what was not saved

Status: accepted

## Context

Settings is not unloaded when it is dismissed. Escape and the window's close
button both reach the same place — `dismiss` hides the window, and the shell's
`CloseRequested` handler hides it too — so the page goes on living with every
field exactly as it was left. Opening the window again shows the same page, not
a fresh one.

That makes keeping unsaved edits the free option. Nothing has to be stored,
nothing has to be restored; a half-typed base URL survives a week of the window
being hidden and shown at no cost whatsoever. It is what the code already does.

It is also, unusually for a free option, the wrong one. What survives is
invisible while it survives. Someone who typed a key, was called away, and hit
Escape has left a key in a window that will show it back to them a day later
with no indication that it was never written, in a window whose whole subject is
what Demysto is going to do next. The gap between "the file says this" and "the
window shows this" is the one thing Settings must never open, because every
other window in Demysto reads the file and not the window: a Run uses the
Provider the file names, and Settings showing a different one is Settings
lying.

Once the window grew a mark for unsaved edits — a dot on the tab that holds them
— the survival stopped being invisible and started being loud. A dot that is
still there tomorrow, on a window reopened from the tray, is either a bug or a
question nobody asked to be asked.

Two alternatives were considered. **Warning on close** — a dialog asking whether
to save first. Rejected because it teaches people not to press Escape, and
Escape closing a window is the one thing this window's keyboard already promised
(`settings-keys`). **Keeping the edits and saying so** — a line in the footer
naming what is unsaved. Rejected because it makes the window's memory a feature
that must then be explained, versioned and reasoned about, in exchange for
recovering typing that took ten seconds.

## Decision

Closing Settings discards everything not saved. The window is put back to the
Settings File as it was last read: Providers and their Models, the Default
Model, the Default Vision Model, the Palette's Hotkey, the size at which a
Selection is called large, the language, the typed key and the request to remove
one, and the Action being edited. Both ways of closing do it, which is why the
frontend listens for the close request rather than leaving it to the shell
alone.

Switching tabs discards nothing. A tab is a place in one window, not another
window, and an Action half-written on one tab is still being written while its
author checks a Model on another.

Autostart is not put back, because it was never held: ticking the box changes
the operating system's login items there and then, and there is nothing for a
close to undo.

## Consequences

The window and the Settings File agree whenever the window is opened. That is
the property this buys, and it is worth more than the typing it costs.

The typing it costs is real. Escape is one key, it is the key this window trains
people to press, and pressing it over a half-configured Provider throws that
Provider away with no confirmation. The unsaved dot is the whole of the warning
anyone gets, which is why the dot has to be legible to a screen reader as well
as visible, and why the Save button stays reachable from every tab.

Nothing needs to be stored for this to work, and nothing needs to be migrated if
it is reversed. What a reversal would have to bring back is not code but an
answer: what a window that remembers unwritten edits should say about them, and
for how long.

# Demysto — an area of the screen

Status: ready for implementation. Vocabulary follows `GLOSSARY.md`; the decisions
recorded in `docs/adr/0001`–`0019` are binding here and are not re-argued. This
builds on `docs/spec/0001` and `docs/spec/0002`, which stand unchanged except
where this document says otherwise. User stories continue their numbering.

## Problem Statement

A picture reaches Demysto only by being copied first. Much of what somebody
wants to ask about was never meant to be copied: a chart inside a PDF viewer, a
frame of a video, a dialog that offers no "copy image", a part of a page wider
than any one element. Copying it means taking a screenshot with the system's own
keys, waiting for it to land on the clipboard, and only then pressing the
Hotkey — three steps where text needs one.

## Solution

A second Hotkey, beside the Palette's. Pressing it lets the user drag a rectangle
over any part of the screen, and the Palette opens on that picture, listing the
Actions that take an image Selection — exactly as if the picture had been copied.
Everything after the Palette is unchanged: fitting and asking for the original
(ADR-0017), the Default Vision Model, Sealed Conversations.

## User Stories

89. As a reader, I want to drag a rectangle over something on screen and ask
    about it, so that a picture nobody can copy is as easy to ask about as one
    somebody can.
90. As a user, I want to change or clear that Hotkey, and the Palette's, so
    that one another application already answers to can be given back.
91. As a user, I want pressing it to leave my clipboard and my selection alone,
    so that asking about the screen does not cost me what I had copied.
92. As a user, I want Esc or a stray click to simply end it, so that changing my
    mind costs nothing and is not reported back to me as an error.
93. As a reader, I want the Palette to say the picture came from the screen, so
    that I can tell it apart from one I had copied a while ago.
94. As a Wayland user, I want the desktop's own screenshot tool to choose the
    area, so that Demysto needs no permission beyond what the desktop asks for.
95. As a user, I want to start an area from the tray, so that I can ask about
    the screen without remembering the Hotkey.

## Implementation Decisions

### The Hotkey

The screen-area Hotkey lives in Settings → General under the Palette's, recorded
with the same control, and is written to the Settings File as
`screen_area_hotkey`. Its default is Ctrl+Shift+Space on macOS, beside the
Palette's Cmd+Shift+Space, and Ctrl+Alt+Space on Windows and Linux; a default
another application holds is reported like the Palette's. Either Hotkey can be
cleared: the Settings File states it as `""`, and a line left out means the
default. Settings offers both clearing and going back to the default. It is
claimed after the Palette's and before any Action's, so an Action
stating the same combination is told the screen area already has it. On Wayland
it is one more shortcut asked of the GlobalShortcuts portal, like the Palette's.

Pressing it is an explicit request for the screen. It ignores whatever text is
selected or copied, sends no copy chord and leaves the clipboard alone.

The tray menu has "Area of the Screen…" under "Actions…", which does what the
Hotkey does. Where the area is unavailable the item is not there.

### Capture

Capture is now the act of obtaining a Selection from the foreground
application, the clipboard or the screen. A picture taken from the screen is an
image Selection like any other; its origin is `screen`, which the Palette shows
as "From the screen".

### The mechanism, per platform

ADR-0019 records why the system's own tool is used wherever there is one:

- **macOS:** `screencapture -i`, writing to a temporary file that is read and
  removed at once, whether the area was chosen or not. Demysto draws nothing.
  It needs Screen Recording for Demysto, which starts it: without it the user
  can still drag, and then no picture is written. So macOS is asked first, at
  every press; where it has not granted it, the press raises the system's own
  prompt, notifies the user naming the pane where it is granted, and opens no
  Palette. The first-run flow names the Hotkey but does not ask: the
  permission is wanted only by somebody who presses it.
- **Linux, where the portal offers Screenshot:** an interactive request to
  `org.freedesktop.portal.Screenshot`, which shows GNOME's or KDE's own UI. The
  file it returns is removed once read. GNOME's UI also puts the picture on the
  clipboard and shows a notification of its own, which Demysto cannot prevent:
  there user story 91 holds for the selection only.
- **Windows, and X11 where the portal offers no Screenshot:** Demysto's own
  overlay, below. On X11 the portal is asked at every press, since it may start
  or stop between them; one that does not answer in time counts as offering it.
- **Wayland where the portal offers no Screenshot:** unavailable. Settings shows
  the field disabled with a sentence saying why and naming GNOME and KDE Plasma,
  which offer it. A portal that cannot be reached within a few seconds counts
  as offering it, and is left to the press to report.

### The overlay

- **A frozen frame.** The screen is captured the moment the Hotkey is pressed,
  and the user selects on that still picture, so that a tooltip or an open menu
  stays in it. On Windows GDI copies it, layered windows included; on X11 it is
  read from the root window, which any client may do without a permission.
- **One monitor:** the one under the pointer. Within it there is one scale, so
  the rectangle is cropped from the frame in that monitor's physical pixels,
  correct at any scaling — Windows' DPI, GDK's window scale on X11 — and on a
  monitor left of or above the primary one. The window covers that monitor
  exactly, full screen on X11 so that no panel keeps it out, and the page
  measures the drag in the frame's own pixels.
- **What it looks like.** The frame is dimmed; the area being dragged shows
  undimmed, with its size in pixels beside it. The cursor is a crosshair.
- **The window.** Created at the press, hidden until the frame is drawn, and
  destroyed afterwards, never kept hidden. It takes the foreground, because it
  needs the mouse and Esc — the one exception to Demysto's windows leaving the
  foreground alone. After a cancel the application that was in front is in
  front again.
- The Palette's Hotkey does nothing while the overlay is up; the screen-area
  Hotkey pressed again cancels it, and so does the overlay losing the focus,
  which leaves in front whatever the user clicked. An overlay that has not
  appeared within a few seconds is a failed tool.
- Only the frame and the foreground belong to the OS. The window and the
  selection are shared, so that X11 without the portal takes the same overlay.

### Cancelling

Esc, a click without a drag and an area of a couple of pixels open nothing and
say nothing: no Palette, no Run, no notification, no Model asked about an empty
picture. The system tool's exit status is not trusted for this; a missing or
empty file, or a cancelled portal request, is the signal. GNOME answers Esc
in its screenshot UI as a request that failed with no reason given, so that
answer counts as a cancel from any portal.

A tool that fails — `screencapture` that will not start, a portal that refuses
or errors — is reported in a notification quoting what it said, on every
platform. The Palette is not opened on it: its retry reads the clipboard, which
is not what was asked for.

## Testing Decisions

The core is tested at the same seam as the clipboard: a fake screen hands over
a file or none, and the suite checks that a picture becomes a Selection from the
screen offered only to image Actions, that a missing, empty or tiny one is a
cancel that changes nothing, and that the clipboard and the foreground
application are never touched. The tools themselves are checked on the Labs: a
Scenario presses the Hotkey and Escape on every Lab and finds no Palette and no
notification, and one that drags an area across a known picture checks the crop
the mock Provider receives.

## Out of Scope

- A per-Action "ask for an area" Hotkey. The screen-area Hotkey opens the
  Palette, which already narrows to image Actions.
- An area across monitors. Moving the pointer to the other monitor before
  pressing is the answer.
- A Palette row that starts the area. The Palette opens over what it would be
  asked to capture.
- Opening the Palette whenever a picture lands on the clipboard. Copying stays
  something the user does, and pressing the Hotkey is how they ask.

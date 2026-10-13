# ADR-0019: The system's own tool chooses an area of the screen, and Demysto's overlay only where there is none

Status: accepted

## Context

The screen-area Hotkey lets somebody drag out a rectangle and opens the Palette
on that picture. Something has to draw the rectangle and hand back the pixels.
Two kinds of thing can: a tool the operating system already has, or an overlay
Demysto draws itself — a frozen frame of the monitor, dimmed, with the dragged
area undimmed.

What the system offers differs by platform:

- **macOS** has `screencapture -i`, the same selection as Cmd+Shift+4, which
  writes the area to a file and writes nothing when cancelled.
- **Linux** has `org.freedesktop.portal.Screenshot` in interactive mode on GNOME
  and KDE, which shows the desktop's own screenshot UI. On Wayland it is the only
  way an application may see the screen at all.
- **Windows** has Snipping Tool, but the API that reports back a picture or a
  cancel needs an MSIX-packaged application, which Demysto is not. The older
  `ms-screenclip:` route only drops the picture on the clipboard and never says
  when the user cancelled, and Snipping Tool may be absent altogether (LTSC,
  Server, or removed).
- **X11 without the portal** — Xfce, i3 and most window managers outside GNOME
  and KDE — has nothing.

## Decision

Wherever the system has a tool, Demysto uses it and draws nothing: `screencapture
-i` on macOS, the portal's Screenshot on Linux where it is offered. Demysto's own
overlay is used only where there is none: on Windows, and on X11 without the
portal. On Wayland without the portal the feature is unavailable, and says so.

## Consequences

The overlay is not used everywhere, though one overlay would look the same on
every platform:

- On macOS it would need Screen Recording, a permission macOS asks the user to
  confirm again every few weeks, for a feature used now and then.
- On Wayland the portal is needed anyway: no client may read the screen without
  it, so an overlay there would still begin with the portal.
- On every platform it means multi-monitor and DPI code — which monitor the
  pointer is on, its scale, a frame cropped in its physical pixels — that the
  system's tool already gets right.

What it costs is two looks for one feature: the system's selection on macOS and
Linux, Demysto's on Windows and bare X11. The overlay is written once and shared
by the two places that need it.

---
status: accepted
---

# On Wayland, capture degrades to clipboard-only

Demysto captures a Selection by sending a synthetic `Cmd/Ctrl+C` to the
foreground application and reading the clipboard. Wayland makes this impossible
by design: XTEST is unavailable to ordinary clients, and injecting input through
the RemoteDesktop portal requires a per-session grant that presents itself to the
user as "this program wants to control your screen". Rather than ask for that, on
Wayland we register the hotkey through the `org.freedesktop.portal.GlobalShortcuts`
portal (KDE, and GNOME from roughly 48) and fall back to reading whatever the
user copied themselves, saying so plainly in Settings.

X11 keeps the full experience. The session type is detected via
`XDG_SESSION_TYPE`.

## Consequences

Linux users on Wayland get a materially worse product than everyone else, and
that is accepted for v1. Pretending otherwise — or pulling the RemoteDesktop
portal in to close the gap — was rejected as worse than the honest limitation.

from _demysto import APP, configured_launch, foreground, nonce, palette_hotkey, say

LAUNCH = False
# What is driven here is Demysto's own overlay, which X11 gets for want of a
# portal offering Screenshot; Wayland takes the desktop's own tool instead.
OS = ["windows", "linux"]
SESSIONS = ["x11"]

STATED = "Ctrl+Alt+Shift+A"


def scenario(g):
    configured_launch(g, settings=f'screen_area_hotkey = "{STATED}"\n')
    in_front = foreground(g)

    clip = "copied before " + nonce()
    g.set_clipboard(clip)

    # Proves an open Palette is seen at all, so that not seeing one below means something.
    g.press(palette_hotkey(g))
    palette = g.wait_for(text=clip, app=APP, timeout=15)
    g.check("the Palette's Hotkey opens a Palette this Scenario can see", palette["met"], detail=palette)
    g.press("escape")
    closed = g.wait_for(text=clip, app=APP, gone=True, timeout=10)
    g.check("Escape closes that Palette", closed["met"], detail=closed)

    staged = g.stage_text("reading here " + nonce())
    reader = staged["app"]

    g.press(STATED.lower())
    up = in_front(APP, 20)
    g.check("the Hotkey brings the overlay to the front", up["met"], detail=up)
    g.screenshot("overlay")

    g.press(palette_hotkey(g))
    ignored = g.wait_for(text=clip, app=APP, timeout=3)
    g.check("the Palette's Hotkey does nothing while the overlay is up", not ignored["met"], detail=ignored)

    g.press("escape")
    back = in_front(reader, 10)
    g.check("Escape gives the foreground back to the application that had it", back["met"], detail=back)

    opened = g.wait_for(text=say(g, "palette-origin-screen"), app=APP, timeout=5)
    g.check("a cancelled area opens no Palette", not opened["met"], detail=opened)
    reopened = g.wait_for(text=clip, app=APP, timeout=2)
    g.check("nor the Palette on the clipboard", not reopened["met"], detail=reopened)

    g.press(STATED.lower())
    again = in_front(APP, 20)
    g.check("the Hotkey brings the overlay up again", again["met"], detail=again)
    g.press(STATED.lower())
    dismissed = in_front(reader, 10)
    g.check("the screen-area Hotkey pressed again closes the overlay", dismissed["met"], detail=dismissed)

    told = g.wait_for(notification=".", timeout=5)
    g.check("a cancelled area notifies nothing", not told["met"], detail=told)
    g.close_staged(staged)

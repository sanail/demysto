import re

from _demysto import APP, HERE, configured_launch, nonce, palette_hotkey, say

LAUNCH = False
# What is driven here is Demysto's own overlay, which the X11 Labs get for
# want of a portal offering Screenshot.
OS = ["windows", "linux"]

STATED = "Ctrl+Alt+Shift+A"


def scenario(g):
    configured_launch(g, settings=f'screen_area_hotkey = "{STATED}"\n')
    if g.os == "windows":
        script = g.put(r"%TEMP%\demysto-front.ps1", (HERE / "mock" / "front.ps1").read_text())
        argv = ["powershell", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", script]
    else:
        argv = ["sh", g.put("/tmp/demysto-front.sh", (HERE / "mock" / "front.sh").read_text())]

    def in_front(process, timeout):
        return g.wait_for(exec=argv, pattern=rf"(?im)^{re.escape(process)}$", timeout=timeout)

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

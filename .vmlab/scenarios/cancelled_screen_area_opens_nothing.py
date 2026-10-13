from _demysto import APP, configured_launch, nonce, palette_hotkey, say

LAUNCH = False
# What is driven here is macOS's own `screencapture -i`.
OS = ["macos"]

STATED = "Ctrl+Alt+Shift+A"


def scenario(g):
    configured_launch(g, settings=f'screen_area_hotkey = "{STATED}"\n')
    clip = "copied before " + nonce()
    g.set_clipboard(clip)

    # Proves an open Palette is seen at all, so that not seeing one below means something.
    g.press(palette_hotkey(g))
    palette = g.wait_for(text=clip, app=APP, timeout=15)
    g.check("the Palette's Hotkey opens a Palette this Scenario can see", palette["met"], detail=palette)
    g.press("escape")
    closed = g.wait_for(text=clip, app=APP, gone=True, timeout=10)
    g.check("Escape closes that Palette", closed["met"], detail=closed)

    g.press(STATED.lower())
    selecting = g.wait_for(process="screencapture", timeout=15)
    g.check("the Hotkey starts the system's area selection", selecting["met"], detail=selecting)
    g.screenshot("area selection")
    g.press("escape")
    ended = g.wait_for(process="screencapture", gone=True, timeout=10)
    g.check("Escape ends the area selection", ended["met"], detail=ended)

    opened = g.wait_for(text=say(g, "palette-origin-screen"), app=APP, timeout=5)
    g.check("a cancelled area opens no Palette", not opened["met"], detail=opened)
    reopened = g.wait_for(text=clip, app=APP, timeout=2)
    g.check("nor the Palette on the clipboard", not reopened["met"], detail=reopened)
    told = g.wait_for(notification=".", timeout=10)
    g.check("a cancelled area notifies nothing", not told["met"], detail=told)
    g.screenshot("after the cancel")

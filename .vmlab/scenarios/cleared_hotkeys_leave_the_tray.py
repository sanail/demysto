from _demysto import APP, area_hotkey, configured_launch, foreground, nonce, palette_hotkey, say, tray

LAUNCH = False
# Whether an area selection is up is read from its process or its window, and
# GNOME's screenshot UI on Wayland is neither one a Guest helper can see.
SESSIONS = ["x11"]


def scenario(g):
    configured_launch(g, settings='palette_hotkey = ""\nscreen_area_hotkey = ""\n')
    clip = "copied before " + nonce()
    g.set_clipboard(clip)
    in_front = None if g.os == "macos" else foreground(g)

    def area_up(timeout):
        if g.os == "macos":
            return g.wait_for(process="screencapture", timeout=timeout)
        return in_front(APP, timeout)

    g.press(palette_hotkey(g))
    opened = g.wait_for(text=clip, app=APP, timeout=5)
    g.check("a cleared Palette Hotkey opens nothing", not opened["met"], detail=opened)
    g.press(area_hotkey(g))
    up = area_up(5)
    g.check("a cleared area Hotkey starts nothing", not up["met"], detail=up)

    g.tray(tray(g), choose=say(g, "tray-area"), timeout=10)
    up = area_up(15)
    g.check("the tray item starts the area selection with its Hotkey cleared", up["met"], detail=up)
    g.screenshot("area from the tray")
    g.press("escape")
    shown = g.wait_for(text=say(g, "palette-origin-screen"), app=APP, timeout=5)
    g.check("Escape ends it with no Palette", not shown["met"], detail=shown)

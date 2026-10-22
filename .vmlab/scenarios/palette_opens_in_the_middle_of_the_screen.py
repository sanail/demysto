from _demysto import APP, configured_launch, nonce, palette_hotkey, check_no_resize, watch_the_palette, work_area

LAUNCH = False
# Where a window goes does not depend on the Lab language.
LANGUAGES = ["en-US"]
# Wayland does not let an application place its own window.
SESSIONS = ["x11"]

# How far the Palette may stray from where it belongs, in logical pixels:
# rounding, and nothing a user would see.
SLACK = 2


def opened_with_pointer_at(g, at, corner):
    """Opens the Palette over a selection with the pointer parked at `at`, and
    returns where its window came up."""
    g.pointer("move", *at)
    watching = watch_the_palette(g)
    text = f"pointer {corner} {nonce()}"
    staged = g.stage_text(text, then=palette_hotkey(g))
    shown = g.wait_for(text=text, app=APP, timeout=15)
    g.check(f"the Palette opens with the pointer in the {corner} corner", shown["met"],
            detail=[staged, shown])
    check_no_resize(g, watching, f"with the pointer in the {corner} corner")
    windows = [m["bounds"] for m in g.find(role="window", app=APP)["matches"] if m.get("bounds")]
    g.screenshot(f"pointer {corner}")
    g.press("escape")
    g.wait_for(text=text, app=APP, gone=True, timeout=10)
    g.close_staged(staged)
    return windows[0] if windows else None


def scenario(g):
    configured_launch(g)
    scale = g.displays()["displays"][0]["scale"]
    x, y, width, height = work_area(g, scale)
    slack = SLACK * scale

    # The two opposite corners, inset past the Dock, the menu bar and the panels. The pointer
    # moves there without a click, which could open what the desktop shows in a corner.
    for at, corner in [((x + width - 40 * scale, y + 40 * scale), "top-right"),
                       ((x + 40 * scale, y + height - 120 * scale), "bottom-left")]:
        bounds = opened_with_pointer_at(g, at, corner) or {"x": None, "y": None, "w": 0, "h": 0}
        expected = (x + (width - bounds["w"]) / 2, y + (height - bounds["h"]) / 2)
        g.check(f"with the pointer in the {corner} corner the Palette is centred on the work area",
                bounds["x"] is not None
                and abs(bounds["x"] - expected[0]) <= slack and abs(bounds["y"] - expected[1]) <= slack,
                detail={"work area": [x, y, width, height], "expected": expected, "bounds": bounds})

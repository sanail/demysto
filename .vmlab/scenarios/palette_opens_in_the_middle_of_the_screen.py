from _demysto import APP, configured_launch, nonce, palette_hotkey

LAUNCH = False
# Where a window goes does not depend on the Lab language.
LANGUAGES = ["en-US"]

# How far the Palette may stray from where it belongs, in the units the tree
# gives bounds in: rounding, and nothing a user would see.
SLACK = 2

# The work area as the units the tree gives bounds in: physical pixels on
# Windows, so the script declares itself DPI-aware or Windows scales its answer.
WINDOWS_WORK_AREA = """
Add-Type -Name Dpi -Namespace Vmlab -MemberDefinition '[DllImport("user32.dll")] public static extern bool SetProcessDPIAware();'
[void][Vmlab.Dpi]::SetProcessDPIAware()
Add-Type -AssemblyName System.Windows.Forms
$a = [System.Windows.Forms.Screen]::PrimaryScreen.WorkingArea
"$($a.X) $($a.Y) $($a.Width) $($a.Height)"
"""

# NSScreen counts from the bottom; the tree counts from the top.
MAC_WORK_AREA = ('ObjC.import("AppKit"); var s = $.NSScreen.mainScreen, f = s.frame, v = s.visibleFrame;'
                 '[v.origin.x, f.size.height - v.origin.y - v.size.height, v.size.width, v.size.height].join(" ")')


def work_area(g):
    """The primary screen less its Dock, menu bar, taskbar or panels: x, y, width, height."""
    if g.os == "windows":
        script = g.put(r"%TEMP%\demysto-work-area.ps1", WINDOWS_WORK_AREA)
        out = g.exec(["powershell", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", script]).stdout
    elif g.os == "macos":
        out = g.exec(["osascript", "-l", "JavaScript", "-e", MAC_WORK_AREA]).stdout
    else:
        # _NET_WORKAREA: x, y, width, height once per desktop.
        out = g.exec(["xprop", "-root", "_NET_WORKAREA"]).stdout.split("=")[1].replace(",", " ")
    return [float(n) for n in out.split()[:4]]


def opened_with_pointer_at(g, at, corner):
    """Opens the Palette over a selection with the pointer parked at `at`, and
    returns where its window came up."""
    g.click(at=at)
    text = f"pointer {corner} {nonce()}"
    staged = g.stage_text(text, then=palette_hotkey(g))
    shown = g.wait_for(text=text, app=APP, timeout=15)
    g.check(f"the Palette opens with the pointer in the {corner} corner", shown["met"],
            detail=[staged, shown])
    windows = [m["bounds"] for m in g.find(role="window", app=APP)["matches"] if m.get("bounds")]
    g.screenshot(f"pointer {corner}")
    g.press("escape")
    g.wait_for(text=text, app=APP, gone=True, timeout=10)
    g.close_staged(staged)
    return windows[0] if windows else None


def scenario(g):
    configured_launch(g)
    x, y, width, height = work_area(g)

    # The two opposite corners, inset past the Dock's ends, the menu bar and
    # the panels, so that the clicks that park the pointer land on nothing.
    for at, corner in [((x + width - 40, y + 40), "top-right"), ((x + 40, y + height - 120), "bottom-left")]:
        bounds = opened_with_pointer_at(g, at, corner) or {"x": None, "y": None, "w": 0}
        expected = (x + (width - bounds["w"]) / 2, y + height / 4)
        g.check(f"with the pointer in the {corner} corner the Palette is centred a quarter of "
                "the way down the work area",
                bounds["x"] is not None
                and abs(bounds["x"] - expected[0]) <= SLACK and abs(bounds["y"] - expected[1]) <= SLACK,
                detail={"work area": [x, y, width, height], "expected": expected, "bounds": bounds})

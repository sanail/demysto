import contextlib

from _demysto import (APP, HERE, MANY_ACTIONS, check_fitted, check_no_resize, check_scrolls_to_last, configured_launch,
                      copy_picture, drawn_bounds, listed, nothing_selected, palette_hotkey, palette_layout, picture_name,
                      say, watch_the_palette, work_area)

LAUNCH = False

# How far past the screen's margin the Palette may reach, in logical pixels: rounding.
SLACK = 2
# Display modes each Guest offers that leave a work area shorter than a picture
# with ten Actions under it, and still taller than the Palette's other rows.
SHORT = {"macos": "1024x576", "windows": "1280x960", "x11": "1024x576", "wayland": "848x480"}

# xrandr knows no mode the Guest's driver does not list, so one is made from gtf.
X11_MODE = """set -e
output=$(xrandr | awk '/ connected/ {print $1; exit}')
current=$(xrandr | awk '/\\*/ {print $1; exit}')
if [ "$1" = restore ]; then xrandr --output "$output" --mode "$2"; exit; fi
set -- $(gtf ${1%x*} ${1#*x} 60 | sed -n 's/^ *Modeline *//p' | tr -d '"')
xrandr --newmode "$@" 2>/dev/null || true
xrandr --addmode "$output" "$1"
xrandr --output "$output" --mode "$1"
echo "$current"
"""


@contextlib.contextmanager
def short_screen(g):
    """The Guest's display at SHORT's mode, and back at its own afterwards."""
    size = SHORT[g.session or g.os]
    if g.os == "macos":
        holder = g.spawn(["osascript", "-l", "JavaScript", g.put("/tmp/demysto-short.js", (HERE / "screen" / "short.js").read_text()), size])
        g.wait_for(log=holder.log, pattern="switched", timeout=15)
        try:
            yield
        finally:
            holder.stop()
    elif g.os == "windows":
        script = g.put(r"%TEMP%\demysto-short.ps1", (HERE / "screen" / "short.ps1").read_text())
        run = ["powershell", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", script]
        switched = g.exec(run + size.split("x"))
        if not switched.ok:
            raise RuntimeError(f"the display stayed as it was: {switched.stderr}")
        try:
            yield
        finally:
            g.exec(run)
    elif g.session == "wayland":
        script = g.put("/tmp/demysto-short.py", (HERE / "screen" / "short-wayland.py").read_text())
        before = g.exec(["python3", script, "current"]).stdout.strip()
        g.exec(["python3", script, size])
        try:
            yield
        finally:
            g.exec(["python3", script, before])
    else:
        script = g.put("/tmp/demysto-short.sh", X11_MODE)
        before = g.exec(["sh", script, size]).stdout.strip()
        try:
            yield
        finally:
            g.exec(["sh", script, "restore", before])


def scenario(g):
    with short_screen(g):
        configured_launch(g, MANY_ACTIONS)
        display = g.displays()["displays"][0]
        scale = display["scale"]
        copy_picture(g, 800, 600)
        staged = nothing_selected(g)
        watching = watch_the_palette(g)
        g.press(palette_hotkey(g))
        shown = g.wait_for(text=picture_name(g), app=APP, timeout=15)
        g.check("the Palette shows the picture", shown["met"], detail=shown)
        check_no_resize(g, watching, "on a short screen,")
        choosing = say(g, "palette-keys-choosing")
        window, footer, actions = palette_layout(g, choosing)
        g.screenshot("short screen")
        check_fitted(g, window, footer, "on a short screen")

        # Wayland tells an application no work area: the screen stands in for it.
        if g.session == "wayland":
            bounds = display["bounds"]
            x, y, width, height = bounds["x"], bounds["y"], bounds["w"], bounds["h"]
        else:
            x, y, width, height = work_area(g, scale)
        margin, drawn = 8 * scale, drawn_bounds(g, window)
        detail = {"work area": [x, y, width, height], "drawn": drawn}
        g.check("the Palette keeps off the top of the work area",
                drawn is not None and drawn["y"] >= y + margin - SLACK * scale, detail=detail)
        if g.session == "wayland":
            g.skip("the Palette keeps off the bottom of the work area",
                   "Wayland tells the Palette neither its work area nor where it is placed, so the height it "
                   "stops at leaves room for no panel, and the window is placed below the top one")
        else:
            g.check("the Palette keeps off the bottom of the work area",
                    drawn is not None and drawn["y"] + drawn["h"] <= y + height - margin + SLACK * scale,
                    detail=detail)
        shows = listed(actions, footer) if footer else []
        g.check("fewer than ten Actions show and the list scrolls to the rest",
                0 < len(shows) < 10 and len(actions) == 14, detail={"listed": shows, "actions": actions})
        check_scrolls_to_last(g, actions, choosing)
        g.press("escape")
        if staged:
            g.close_staged(staged)

import time

from _demysto import (APP, BLOCKS, area_hotkey, configured_launch, decode_png, foreground, mock, nearest_block, nonce,
                      received, say, show_blocks, shows_answer, check_no_resize, watch_the_palette)

LAUNCH = False

# The block dragged across: the top middle one, away from every panel and menu bar.
TARGET = 1


def scenario(g):
    answer = "Mock description " + nonce()
    provider = mock(g, answer)
    wayland = g.session == "wayland"
    # The default rather than a stated combination, so that the drag also proves
    # nothing on a stock desktop holds it.
    chord = area_hotkey(g)
    configured_launch(g)

    screen = show_blocks(g)
    x, y, w, h = screen["x"], screen["y"], screen["w"], screen["h"]
    in_front = foreground(g) if g.os in ("windows", "linux") and not wayland else None

    def tool_up():
        if g.os == "macos":
            return g.wait_for(process="screencapture", timeout=15)
        if wayland:
            # GNOME's screenshot UI shows nothing a Guest helper can read, so it
            # is given the time it takes to come up; the drag below proves it does.
            time.sleep(3)
            return None
        return in_front(APP, 20)

    def nothing_opened(how):
        opened = g.wait_for(text=say(g, "palette-origin-screen"), app=APP, timeout=5)
        g.check(f"{how} opens no Palette", not opened["met"], detail=opened)

    g.press(chord)
    up = tool_up()
    if up is None:
        g.skip("the Hotkey brings up the area selection", "GNOME's screenshot UI cannot be seen from the Guest")
    else:
        g.check("the Hotkey brings up the area selection", up["met"], detail=up)
    g.press("escape")
    nothing_opened("Escape")

    g.press(chord)
    tool_up()
    # In the red block, where the drag below does not go.
    g.click(at=(x + w // 6, y + h // 4))
    nothing_opened("a click without a drag")
    if wayland:
        # A click alone does not end GNOME's selection; Escape does.
        g.press("escape")

    clip = "copied before " + nonce()
    g.set_clipboard(clip)
    watching = watch_the_palette(g)
    g.press(chord)
    tool_up()
    start = (x + w // 3 + w // 12, y + h // 8)
    end = (x + 2 * w // 3 - w // 12, y + 3 * h // 8)
    g.drag(start, end)
    if wayland:
        g.press("enter")
    shown = g.wait_for(text=say(g, "palette-origin-screen"), app=APP, timeout=15)
    g.check("the dragged area opens the Palette from the screen", shown["met"], detail=shown)
    check_no_resize(g, watching, "on a picture")
    g.screenshot("palette on the area")

    g.type(say(g, "action-describe-image-name"))
    g.press("enter")
    answered, seen = shows_answer(g, answer)
    g.check("Describe image shows the vision Model's answer", answered, detail=[seen, provider.output()[-500:]])
    g.press("escape")

    sent = received(provider)
    g.check("the Model is sent one picture", len(sent) == 1, detail=len(sent))
    if not sent:
        return
    width, height, rows = decode_png(sent[0])
    wanted = (end[0] - start[0], end[1] - start[1])
    g.check("the picture is the size of the dragged area",
            abs(width - wanted[0]) <= 2 and abs(height - wanted[1]) <= 2, detail={"got": (width, height), "dragged": wanted})
    # Two pixels in from every edge, where the selection's own edge may fall.
    inside = [pixel for row in rows[2:-2] for pixel in row[2:-2]]
    stray = [pixel for pixel in inside if nearest_block(pixel) != TARGET]
    g.check("the picture is the dragged block's colour all over", inside and not stray,
            detail={"block": BLOCKS[TARGET], "stray": len(stray), "of": len(inside), "first": stray[:5]})

    if wayland:
        g.skip("the clipboard still holds what was copied before",
               "GNOME's screenshot UI puts every picture it takes on the clipboard itself")
    else:
        kept = g.clipboard()
        g.check("the clipboard still holds what was copied before", kept.get("text") == clip, detail=kept)
    # Since the Run began, so the cancels above are covered too.
    told = g.wait_for(notification=".", timeout=3)
    g.check("neither the cancels nor the area notify anything", not told["met"], detail=told)

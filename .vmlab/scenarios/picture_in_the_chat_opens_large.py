import collections
import time

from _demysto import (APP, configured_launch, copy_picture, mock, nonce, nothing_selected, palette_hotkey, pictures,
                      say, screen_pixels, shows_answer)

LAUNCH = False

# Larger than the 1568-pixel ceiling, so the fitted picture and the original differ.
WIDTH, HEIGHT = 2000, 1200
FITTED = (1568, 941)
# The open picture's padding, in logical pixels.
PADDING = 24


def picture_bounds(rows, area):
    """The bounds of what stands out from the commonest colour inside area:
    the open picture, on the background it leaves above and below it. A few
    pixels in from the edges, where Windows draws the window's border."""
    inset = 4
    x0, y0 = int(area["x"]) + inset, int(area["y"]) + inset
    region = [row[x0:int(area["x"] + area["w"]) - inset] for row in rows[y0:int(area["y"] + area["h"]) - inset]]
    background = collections.Counter(pixel for row in region for pixel in row).most_common(1)[0][0]

    def stands_out(pixel):
        return sum(abs(a - b) for a, b in zip(pixel, background)) > 48

    ys = [y for y, row in enumerate(region) if sum(map(stands_out, row)) > len(row) // 4]
    xs = [x for x in range(len(region[0])) if sum(stands_out(row[x]) for row in region) > len(region) // 4]
    if not xs or not ys:
        return None
    return {"x": x0 + xs[0], "y": y0 + ys[0], "w": xs[-1] - xs[0] + 1, "h": ys[-1] - ys[0] + 1}


def poll(read, done, timeout):
    """read() until done(what it read) or timeout seconds pass; answers the
    last read. For the focus, which wait_for cannot wait on."""
    deadline = time.monotonic() + timeout
    while True:
        seen = read()
        if done(seen) or time.monotonic() > deadline:
            return seen
        time.sleep(0.3)


def open_picture(g, thumbnail, name):
    """Clicks the picture in the chat and checks it opens fitted to the
    window; answers its pixels, or None when it does not open."""
    g.click(text=thumbnail["name"], role="button", app=APP)
    opened = g.wait_for(text=say(g, "result-picture-enlarged-label"), app=APP, timeout=10)
    g.check(f"{name}, a click on the picture opens it", opened["met"], detail=opened)
    if not opened["met"]:
        return None
    window = opened["matches"][0]["bounds"]
    # Out of the window, so that the pointer is in no screenshot of the picture.
    g.pointer("move", max(0, window["x"] - 40), max(0, window["y"] - 40))
    _, _, rows = screen_pixels(g, name)
    scale = g.displays()["displays"][0]["scale"]
    picture = picture_bounds(rows, window)
    # As wide as the window less its padding, since the picture is wider in
    # shape than the space it has, and as tall as its shape then makes it.
    width = window["w"] - 2 * PADDING * scale
    slack = 3 * scale
    g.check(f"{name}, the open picture is fitted to the window",
            picture is not None and abs(picture["w"] - width) <= slack
            and abs(picture["h"] - width * HEIGHT / WIDTH) <= slack
            and abs(picture["x"] - window["x"] - (window["w"] - picture["w"]) / 2) <= slack
            and abs(picture["y"] - window["y"] - (window["h"] - picture["h"]) / 2) <= slack,
            detail={"window": window, "picture": picture, "width": width})
    return [row[int(picture["x"]):int(picture["x"] + picture["w"])]
            for row in rows[int(picture["y"]):int(picture["y"] + picture["h"])]] if picture else None


def check_closed(g, thumbnail, how):
    """Checks the open picture is gone and the focus is back on its thumbnail."""
    left = g.wait_for(text=say(g, "result-picture-enlarged-label"), app=APP, gone=True, timeout=10)
    g.check(f"{how} closes the open picture", left["met"], detail=left)
    # The tree may lag behind the focus by a moment.
    focused = poll(lambda: [m for m in g.find(text=thumbnail["name"], role="button", app=APP)["matches"]
                            if m.get("focused")], bool, 5)
    g.check(f"closed by {how}, the focus goes back to the picture", focused != [], detail=focused)


def differing(before, after):
    """The share of pixels two pictures of one size differ in."""
    if before is None or after is None or len(before) != len(after) or len(before[0]) != len(after[0]):
        return None
    pairs = [(a, b) for row_a, row_b in zip(before, after) for a, b in zip(row_a, row_b)]
    return sum(1 for a, b in pairs if sum(abs(x - y) for x, y in zip(a, b)) > 24) / len(pairs)


def scenario(g):
    answer = "Mock description " + nonce()
    provider = mock(g, answer)
    configured_launch(g)
    copy_picture(g, WIDTH, HEIGHT)

    staged = nothing_selected(g)
    g.press(palette_hotkey(g))
    shown = g.wait_for(text=say(g, "palette-picture", dimensions=f"{WIDTH} × {HEIGHT}"), app=APP, timeout=15)
    g.check("the Palette shows the copied picture", shown["met"], detail=shown)
    g.type(say(g, "action-describe-image-name"))
    g.press("enter")
    answered, seen = shows_answer(g, answer)
    g.check("Describe image shows the vision Model's answer", answered, detail=[seen, provider.output()[-500:]])

    # The chat quotes a picture by its dimensions, which name its thumbnail.
    found = g.wait_for(text=f"{WIDTH} × {HEIGHT}", role="button", app=APP, timeout=10)
    g.check("the chat shows the picture", found["met"], detail=found)
    if not found["met"]:
        return
    thumbnail = found["matches"][0]
    before = open_picture(g, thumbnail, "fitted")
    if before is None:
        return
    g.click(at=(thumbnail["bounds"]["x"] + thumbnail["bounds"]["w"] // 2,
                thumbnail["bounds"]["y"] + thumbnail["bounds"]["h"] // 2))
    check_closed(g, thumbnail, "a click")

    g.click(text=say(g, "result-ask-at-original").split("{ $")[0].rstrip(" —"), role="button", app=APP)
    asked = g.wait_for(log=provider.log, pattern=r"(?s)POST /v1/chat.*POST /v1/chat", timeout=30)
    sent = pictures(provider)
    g.check("asking again sends the original", sent == [FITTED, (WIDTH, HEIGHT)], detail=[sent, asked])
    answered, seen = shows_answer(g, answer)
    g.check("asking again shows the answer", answered, detail=seen)

    after = open_picture(g, thumbnail, "original")
    # Shown at one size, the original resamples to other pixels than the
    # fitted picture does; the same picture would draw the same pixels.
    changed = differing(before, after)
    g.check("after asking at the original, the open picture is the original, not the fitted one",
            changed is not None and changed > 0.01, detail=changed)

    g.press("escape")
    check_closed(g, thumbnail, "Escape")
    kept, seen = shows_answer(g, answer, timeout=5)
    g.check("Escape leaves the window open", kept, detail=seen)

    g.press("escape")
    gone = g.wait_for(text=answer, app=APP, gone=True, timeout=10)
    g.check("a second Escape closes the window", gone["met"], detail=gone)
    if staged:
        g.close_staged(staged)

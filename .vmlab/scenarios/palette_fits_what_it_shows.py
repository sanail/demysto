from _demysto import (APP, MANY_ACTIONS, check_fitted, check_no_resize, check_scrolls_to_last, configured_launch,
                      copy_picture, ink, listed, nonce, nothing_selected, palette_hotkey, palette_layout, picture_name,
                      poll, say, screen_pixels, watch_the_palette, work_area)

LAUNCH = False

# How far the Palette may stray from the middle of the work area, in logical pixels.
SLACK = 2

# The share of a window's pixels that stand out once the Palette is drawn in it.
DRAWN = 0.005

# Wider than the Palette's picture box by far more than it is tall, so that
# shown whole it leaves a band above and below it; cropped, it would fill the box.
PICTURE = (2000, 250)


def open_on(g, text=None):
    """Presses the Palette's Hotkey over a Selection of `text`, or over
    nothing selected so that the Palette takes the clipboard; returns the
    Staged document, if any, and the watcher started before the press."""
    watching = watch_the_palette(g)
    if text and g.session == "wayland":
        # Wayland lets no application copy from another: the Palette reads the clipboard.
        g.set_clipboard(text)
    elif text:
        return g.stage_text(text, then=palette_hotkey(g)), watching
    staged = None if text else nothing_selected(g)
    g.press(palette_hotkey(g))
    return staged, watching


def check_centred(g, window, what):
    if g.session == "wayland":
        return g.skip(f"{what}, the Palette is centred on the work area", "Wayland places the window itself")
    scale = g.displays()["displays"][0]["scale"]
    x, y, width, height = work_area(g, scale)
    expected = (x + (width - window["w"]) / 2, y + (height - window["h"]) / 2) if window else None
    g.check(f"{what}, the Palette is centred on the work area",
            window is not None and abs(window["x"] - expected[0]) <= SLACK * scale
            and abs(window["y"] - expected[1]) <= SLACK * scale,
            detail={"work area": [x, y, width, height], "expected": expected, "window": window})


def scenario(g):
    configured_launch(g, MANY_ACTIONS)
    choosing = say(g, "palette-keys-choosing")

    # The first open after the launch: on Linux without hardware acceleration a
    # window shown in its first moments may come up blank.
    text = "first open " + nonce()
    staged, watching = open_on(g, text)
    shown = g.wait_for(text=text, app=APP, timeout=15)
    g.check("the first open after the launch shows the Selection", shown["met"], detail=shown)
    # Watched for a while from the moment it shows: opened while its page still
    # loads, a Palette may show an older frame at first; one not drawn stays blank.
    window = palette_layout(g, choosing)[0]
    drawn = poll(lambda: ink(screen_pixels(g, "first open")[2], window), lambda seen: seen > DRAWN, 3) if window else 0
    g.check("the first open after the launch draws the Palette", drawn > DRAWN, detail={"ink": drawn, "window": window})
    check_no_resize(g, watching, "on the first open,")
    window, footer, actions = palette_layout(g, choosing)
    check_fitted(g, window, footer, "on a text Selection")
    check_centred(g, window, "on a text Selection")
    shows = listed(actions, footer) if footer else []
    g.check("ten of fifteen Actions show and the rest are below the list",
            len(actions) == 15 and len(shows) == 10, detail={"listed": shows, "actions": actions})
    check_scrolls_to_last(g, actions, choosing)

    # The pointer moved onto an Action chooses it, as the arrows do. Down goes
    # round to the first Action, scrolling back to the top of the list.
    translate = say(g, "action-translate-name")
    asking = say(g, "action-translate-target-label")
    g.press("down")
    _, footer, actions = palette_layout(g, choosing)
    row = actions.get(translate)
    if row:
        # Two moves: a pointer already resting on the row would not have moved onto it.
        g.pointer("move", row["x"] + 10, row["y"] + row["h"] / 2)
        g.pointer("move", row["x"] + row["w"] / 2, row["y"] + row["h"] / 2)
    g.press("enter")
    pointed = g.wait_for(text=asking, app=APP, timeout=5)
    g.check("Enter runs the Action the pointer moved onto", pointed["met"], detail={"row": row, "asked": pointed})
    g.press("escape")
    g.wait_for(text=asking, app=APP, gone=True, timeout=5)

    # Typing reaches the filter, so the Palette holds the focus; filtering and
    # collecting a Parameter leave room rather than resize it.
    watching = watch_the_palette(g)
    g.type(translate)
    narrowed = g.wait_for(text=say(g, "action-explain-name"), role="button", app=APP, gone=True, timeout=5)
    g.check("what is typed goes to the Palette's filter", narrowed["met"], detail=narrowed)
    g.press("enter")
    asked = g.wait_for(text=asking, app=APP, timeout=5)
    g.check("Enter asks for the Translate Parameter", asked["met"], detail=asked)
    check_no_resize(g, watching, "filtering and collecting a Parameter,")
    g.press("escape")
    g.press("escape")
    g.wait_for(text=text, app=APP, gone=True, timeout=10)
    if staged:
        g.close_staged(staged)

    copy_picture(g, *PICTURE)
    staged, watching = open_on(g)
    picture = g.wait_for(text=picture_name(g), app=APP, timeout=15)
    g.check("the Palette shows the picture", picture["met"], detail=picture)
    check_no_resize(g, watching, "on a picture,")
    window, footer, actions = palette_layout(g, choosing)
    check_fitted(g, window, footer, "on a picture")
    check_centred(g, window, "on a picture")
    shows = listed(actions, footer) if footer else []
    g.check("ten of fourteen Actions show under the picture", len(actions) == 14 and len(shows) == 10,
            detail={"listed": shows, "actions": actions})

    # The box keeps one height; inside it, the picture whole leaves a band of
    # background above it and is striped across its whole width.
    scale = g.displays()["displays"][0]["scale"]
    box = picture["matches"][0]["bounds"] if picture["met"] else None
    _, _, rows = screen_pixels(g, "picture")
    middle = rows[int(box["y"] + box["h"] / 2)][int(box["x"]):int(box["x"] + box["w"])] if box else []
    band = rows[int(box["y"] + 10 * scale)][int(box["x"]):int(box["x"] + box["w"])] if box else []
    edges = middle[:int(4 * scale)] + middle[-int(4 * scale):]
    g.check("the picture's box is 160 high", box is not None and abs(box["h"] - 160 * scale) <= SLACK * scale,
            detail=box)
    g.check("the picture shows whole, across the box's width and short of its height",
            bool(band) and len(set(band)) <= 3 and len(set(middle)) > 50 and len(set(edges)) > 4,
            detail={"box": box, "band colours": len(set(band)), "middle colours": len(set(middle)),
                    "edge colours": len(set(edges))})

    g.type("Extra")
    narrowed = g.wait_for(text=say(g, "action-describe-image-name"), role="button", app=APP, gone=True, timeout=5)
    g.check("on a picture, what is typed goes to the Palette's filter", narrowed["met"], detail=narrowed)
    g.press("escape")
    if staged:
        g.close_staged(staged)

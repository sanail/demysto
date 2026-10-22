from _demysto import APP, CATALOGUE_LANGUAGES, approve_shortcuts, configure, largest_window, nodes, say, tray

LAUNCH = False
TIMEOUT = 1200
# An en-US Lab measures every catalogue; another Lab language adds nothing.
LANGUAGES = ["en-US"]

LABELS = ["settings-default-model", "settings-default-vision-model"]
SLACK = 1  # a logical pixel of rounding between how the toolkit and the window report bounds


def fields(g, lang):
    """Each Default Model field's label text and dropdown bounds, in LABELS order.
    WebKitGTK exposes no node for the label text and names the dropdown by its
    label and its value; there the label text is None."""
    shown = [n for n in nodes(g.tree(app=APP)) if n.get("bounds")]
    found = []
    for key in LABELS:
        label = say(g, key, lang)
        dropdown = next((n["bounds"] for n in shown if n["role"] == "combobox"
                         and (n.get("name") or "").startswith(label)), None)
        text = next((n["bounds"] for n in shown if n["role"] != "combobox"
                     and label in (n.get("name"), n.get("value"))), None)
        found.append((text, dropdown))
    return found


def scenario(g):
    configure(g)
    slack = SLACK * g.displays()["displays"][0]["scale"]
    # Level dropdowns prove nothing unless narrowing the window wraps one label and not the other.
    uneven, measured = [], False
    for lang in CATALOGUE_LANGUAGES:
        g.quit()
        g.launch(env={"DEMYSTO_LANGUAGE": lang})
        approve_shortcuts(g)
        g.tray(tray(g), choose=say(g, "tray-settings", lang), timeout=15)
        opened = g.wait_for(text=say(g, LABELS[0], lang), role="combobox", app=APP, timeout=15)
        if not g.check(f"{lang}: Settings opens on its Models", opened["met"], detail=opened):
            continue
        wide = fields(g, lang)
        win = largest_window(g.tree(app=APP))
        # GNOME on Wayland counts the window's shadow, 26 pixels, in its bounds and takes the
        # resize grab just outside the visible frame.
        grab = 23 if g.session == "wayland" else 2
        corner = (win["x"] + win["w"] - grab, win["y"] + win["h"] - grab)
        # Dragged far past the window's minimum width, which stops it there.
        g.drag(corner, (win["x"] + win["w"] // 4, corner[1]))
        narrow = fields(g, lang)
        g.screenshot(f"{lang} narrow")
        for width, found in (("wide", wide), ("narrow", narrow)):
            (_, text_dropdown), (_, pictures_dropdown) = found
            if g.check(f"{lang}, {width}: both Default Model dropdowns are shown",
                       bool(text_dropdown and pictures_dropdown), detail=found):
                g.check(f"{lang}, {width}: the Default Model dropdowns sit level",
                        abs(text_dropdown["y"] - pictures_dropdown["y"]) <= slack, detail=found)
        (text_label, _), (pictures_label, _) = narrow
        if text_label and pictures_label:
            measured = True
            if abs(text_label["h"] - pictures_label["h"]) > slack:
                uneven.append(lang)
    wraps = "narrowing the window wraps one Default Model label and not the other in some language"
    if measured:
        g.check(wraps, bool(uneven), detail=uneven)
    else:
        # Each grid cell is as tall as the taller label, so the dropdowns cannot tell either.
        g.skip(wraps, "the toolkit exposes no label text to measure")

import pathlib

from _demysto import APP, CATALOGUES, configure, language, say, tray

LAUNCH = False
TIMEOUT = 1200

# Every interface language Demysto has a catalogue for.
LANGUAGES = sorted(p.stem for p in pathlib.Path(CATALOGUES).glob("*.ftl"))
TABS = ["settings-tab-models", "settings-tab-actions", "settings-tab-general", "settings-tab-about"]
CONTROLS = {"button", "tab", "radiobutton", "textfield", "textarea", "combobox", "checkbox"}
SLACK = 1  # a pixel of rounding between how the toolkit and the window report bounds


def nodes(tree):
    stack = [tree]
    while stack:
        n = stack.pop()
        yield n
        stack.extend(n.get("children") or [])


def visible(b, win):
    return b and b["w"] > 1 and b["h"] > 1 and b["y"] < win["y"] + win["h"] and b["y"] + b["h"] > win["y"]


def misfits(g, lang):
    """What sticks out of the Settings window sideways, and which controls overlap."""
    tree = g.tree(app=APP)
    windows = [n for n in nodes(tree) if n["role"] == "window" and n.get("bounds")]
    win = max(windows, key=lambda n: n["bounds"]["w"] * n["bounds"]["h"])["bounds"]
    shown = [n for n in nodes(tree) if n["role"] not in ("desktop", "application", "window", "menuitem", "menubar", "menu")
             and visible(n.get("bounds"), win)]
    outside = [f'{n["role"]} {n.get("name") or n.get("value")!r} {n["bounds"]}' for n in shown
               if n["bounds"]["x"] < win["x"] - SLACK
               or n["bounds"]["x"] + n["bounds"]["w"] > win["x"] + win["w"] + SLACK]
    controls = [n for n in shown if n["role"] in CONTROLS]
    overlaps = []
    for i, a in enumerate(controls):
        for b in controls[i + 1:]:
            ab, bb = a["bounds"], b["bounds"]
            w = min(ab["x"] + ab["w"], bb["x"] + bb["w"]) - max(ab["x"], bb["x"])
            h = min(ab["y"] + ab["h"], bb["y"] + bb["h"]) - max(ab["y"], bb["y"])
            # One control drawn inside another (an option inside its list) is not an overlap.
            inside = (w, h) in ((ab["w"], ab["h"]), (bb["w"], bb["h"]))
            if w > SLACK and h > SLACK and not inside:
                overlaps.append(f'{a.get("name")!r} {ab} / {b.get("name")!r} {bb}')
    return outside, overlaps


def scenario(g):
    configure(g)
    tab_role = "radiobutton" if g.os == "macos" else "tab"
    # Every language once, on the en-US Labs; a Lab in another language checks
    # only its own, rather than the same five again.
    languages = LANGUAGES if g.language == "en-US" else [language(g)]
    for lang in languages:
        g.quit()
        g.launch(env={"DEMYSTO_LANGUAGE": lang})
        g.tray(tray(g), choose=say(g, "tray-settings", lang), timeout=15)
        opened = g.wait_for(text=say(g, "settings-tab-models", lang), role=tab_role, app=APP, timeout=15)
        if not g.check(f"{lang}: Settings opens in that language", opened["met"], detail=opened):
            continue
        for key in TABS:
            tab = say(g, key, lang)
            g.click(text=tab, role=tab_role, app=APP, timeout=10)
            if key == "settings-tab-actions":
                g.click(text=say(g, "settings-write-action", lang), role="button", app=APP, timeout=10)
                g.wait_for(text=say(g, "settings-action-accepts", lang), app=APP, timeout=10)
            g.screenshot(f"{lang} {tab}")
            outside, overlaps = misfits(g, lang)
            g.check(f"{lang}, {tab}: nothing sticks out of the window sideways", not outside, detail=outside)
            g.check(f"{lang}, {tab}: no two controls overlap", not overlaps, detail=overlaps)

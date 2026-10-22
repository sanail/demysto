from _demysto import APP, CATALOGUE_LANGUAGES, approve_shortcuts, configure, largest_window, nodes, say, tray

LAUNCH = False
TIMEOUT = 1200
# An en-US Lab measures every catalogue; another Lab language adds nothing.
LANGUAGES = ["en-US"]

TABS = ["settings-tab-models", "settings-tab-actions", "settings-tab-general", "settings-tab-about"]
CONTROLS = {"button", "tab", "radiobutton", "textfield", "textarea", "combobox", "checkbox"}
SLACK = 1  # a logical pixel of rounding between how the toolkit and the window report bounds


def visible(b, win, slack):
    return b and b["w"] > slack and b["h"] > slack and b["y"] < win["y"] + win["h"] and b["y"] + b["h"] > win["y"]


def misfits(g, lang, slack):
    """What sticks out of the Settings window sideways, and which controls overlap."""
    tree = g.tree(app=APP)
    win = largest_window(tree)
    shown = [n for n in nodes(tree) if n["role"] not in ("desktop", "application", "window", "menuitem", "menubar", "menu")
             and visible(n.get("bounds"), win, slack)]
    outside = [f'{n["role"]} {n.get("name") or n.get("value")!r} {n["bounds"]}' for n in shown
               if n["bounds"]["x"] < win["x"] - slack
               or n["bounds"]["x"] + n["bounds"]["w"] > win["x"] + win["w"] + slack]
    controls = [n for n in shown if n["role"] in CONTROLS]
    overlaps = []
    for i, a in enumerate(controls):
        for b in controls[i + 1:]:
            ab, bb = a["bounds"], b["bounds"]
            w = min(ab["x"] + ab["w"], bb["x"] + bb["w"]) - max(ab["x"], bb["x"])
            h = min(ab["y"] + ab["h"], bb["y"] + bb["h"]) - max(ab["y"], bb["y"])
            # One control drawn inside another (an option inside its list) is not an overlap.
            inside = (w, h) in ((ab["w"], ab["h"]), (bb["w"], bb["h"]))
            if w > slack and h > slack and not inside:
                overlaps.append(f'{a.get("name")!r} {ab} / {b.get("name")!r} {bb}')
    return outside, overlaps


def scenario(g):
    # Stated rather than left to the defaults, so that each Hotkey row draws
    # both its clear and its restore buttons, its widest form.
    configure(g, settings='palette_hotkey = "Ctrl+Alt+Shift+P"\nscreen_area_hotkey = "Ctrl+Alt+Shift+A"\n')
    slack = SLACK * g.displays()["displays"][0]["scale"]
    tab_role = "radiobutton" if g.os == "macos" else "tab"
    for lang in CATALOGUE_LANGUAGES:
        g.quit()
        g.launch(env={"DEMYSTO_LANGUAGE": lang})
        approve_shortcuts(g)
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
            outside, overlaps = misfits(g, lang, slack)
            g.check(f"{lang}, {tab}: nothing sticks out of the window sideways", not outside, detail=outside)
            g.check(f"{lang}, {tab}: no two controls overlap", not overlaps, detail=overlaps)

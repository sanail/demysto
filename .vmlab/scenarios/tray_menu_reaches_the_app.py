from _demysto import APP, configured_launch, language, nonce, say, tray

LAUNCH = False


def scenario(g):
    configured_launch(g)
    window = g.wait_for(role="window", app=APP, timeout=5)
    g.check("a launch past the first run opens no window", not window["met"], detail=window)

    items = g.tray(tray(g))["items"]
    names = [item["name"] for item in items]
    for key in ["tray-open", "tray-settings", "tray-quit"]:
        g.check(f"the tray menu has {say(g, key)}", say(g, key) in names, detail=names)
    g.check("the tray menu's first item is the list of Actions", names[:1] == [say(g, "tray-open")], detail=names)
    g.check("the tray menu has no submenu", not any(item["children"] for item in items), detail=items)

    text = "Ohm's law relates voltage and current " + nonce()
    staged = g.stage_text(text)
    g.tray(tray(g), choose=say(g, "tray-open"))
    palette = g.wait_for(text=text, app=APP, timeout=15)
    g.check("the first item opens the Palette on the foreground app's selection", palette["met"],
            detail=[palette, staged])
    g.screenshot("palette from the tray")
    g.press("escape")
    closed = g.wait_for(text=text, app=APP, gone=True, timeout=10)
    g.check("Escape closes the Palette opened from the tray", closed["met"], detail=closed)
    g.close_staged(staged)

    g.tray(tray(g), choose=say(g, "tray-settings"))
    settings = g.wait_for(text=say(g, "settings-add-provider"), app=APP, timeout=15)
    g.check("Settings… opens Settings", settings["met"], detail=settings)

    # A language other than the Lab's (English or Russian), chosen and saved in
    # Settings. Picked by keys, from "Follow the system" down the list (English,
    # Deutsch, ...): the list a select opens is not in the app's tree on every OS.
    other, steps = ("en", 1) if language(g) == "ru" else ("de", 2)
    tab_role = "radiobutton" if g.os == "macos" else "tab"
    g.click(role=tab_role, text=say(g, "settings-tab-general"), app=APP, timeout=10)
    g.click(role="combobox", text=say(g, "settings-language-follows-system"), app=APP, timeout=10)
    for _ in range(steps):
        g.press("down")
    g.press("return")
    g.click(role="button", text=say(g, "settings-save"), app=APP, timeout=10)
    saved = g.wait_for(text=say(g, "settings-save", other), role="button", app=APP, timeout=15)
    g.check("Settings speaks the language saved", saved["met"], detail=saved)
    g.screenshot("settings in the other language")
    renamed = [item["name"] for item in g.tray(tray(g), timeout=15)["items"]]
    g.check("saving another language renames the tray items without a restart",
            renamed == [say(g, key, other) for key in ["tray-open", "tray-settings", "tray-quit"]], detail=renamed)

    g.tray(tray(g), choose=say(g, "tray-quit", other))
    gone = g.wait_for(process=APP, gone=True, timeout=15)
    g.check("Quit Demysto ends the app", gone["met"], detail=gone)

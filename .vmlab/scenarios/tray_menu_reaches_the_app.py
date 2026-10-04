from _demysto import APP, configured_launch, nonce, say, tray

LAUNCH = False


def scenario(g):
    configured_launch(g)
    window = g.wait_for(role="window", app=APP, timeout=5)
    g.check("a launch past the first run opens no window", not window["met"], detail=window)

    items = g.tray(tray(g))["items"]
    names = [item["name"] for item in items]
    for key in ["tray-open", "tray-actions", "tray-settings", "tray-quit"]:
        g.check(f"the tray menu has {say(g, key)}", say(g, key) in names, detail=names)
    actions = next((item["children"] for item in items if item["name"] == say(g, "tray-actions")), [])
    expected = [say(g, f"action-{a}-name") for a in ("explain", "describe-image", "translate", "summarize")]
    g.check("the tray menu's Actions submenu lists the Actions",
            [a["name"] for a in actions][:4] == expected, detail=actions)

    text = "Ohm's law relates voltage and current " + nonce()
    staged = g.stage_text(text)
    g.tray(tray(g), choose=say(g, "tray-open"))
    palette = g.wait_for(text=text, app=APP, timeout=15)
    g.check("Open Demysto opens the Palette on the foreground app's selection", palette["met"],
            detail=[palette, staged])
    g.screenshot("palette from the tray")
    g.press("escape")
    closed = g.wait_for(text=text, app=APP, gone=True, timeout=10)
    g.check("Escape closes the Palette opened from the tray", closed["met"], detail=closed)
    g.close_staged(staged)

    g.tray(tray(g), choose=say(g, "tray-settings"))
    settings = g.wait_for(text=say(g, "settings-add-provider"), app=APP, timeout=15)
    g.check("Settings… opens Settings", settings["met"], detail=settings)

    g.tray(tray(g), choose=say(g, "tray-quit"))
    gone = g.wait_for(process=APP, gone=True, timeout=15)
    g.check("Quit Demysto ends the app", gone["met"], detail=gone)

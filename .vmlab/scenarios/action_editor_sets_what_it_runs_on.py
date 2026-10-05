from _demysto import APP, config_file, configured_launch, copy_picture, nonce, palette_hotkey, say, tray

LAUNCH = False
# What this measures is the same in every Lab language.
LANGUAGES = ["en-US"]


def tab_to(g, label, role=None, presses=15):
    """Tabs forward until the element is focused. The editor is longer than its
    window, and focus scrolls the page to what it lands on, where a click could
    not reach. Returns whether it got there."""
    for _ in range(presses):
        # Without a role: any control named so, not its label (the Prompt is a
        # textarea to macOS, a textfield to Linux, a group to Windows).
        found = g.find(text=label, role=role, app=APP)["matches"]
        if any(m["focused"] and m["role"] != "text" for m in found):
            return True
        g.press("tab")
    return False


def typed(g, field, text):
    """Types into the focused field, and checks the field holds all of it."""
    g.type(text)
    held = g.wait_for(text=text, app=APP, timeout=10)
    g.check(f"the {field} field holds what was typed", held["met"], detail=held)


def toggle(g, label, ticked):
    """Space on the focused tick, and checks the tick took it."""
    g.press("space")
    took = g.wait_for(text=label, role="checkbox", app=APP, checked=ticked, timeout=5)
    g.check(f"{label} is {'ticked' if ticked else 'cleared'}", took["met"], detail=took)


def scenario(g):
    configured_launch(g)
    name = "Read the sign " + nonce()
    prompt = "What does this sign say " + nonce()

    g.tray(tray(g), choose=say(g, "tray-settings"), timeout=10)
    # Settings opens behind whatever is in front; on macOS a first click on a
    # window of an app that is not active only activates it.
    g.wait_for(text=say(g, "settings-add-provider"), app=APP, timeout=15)
    g.focus(APP)
    # The tab is a radio button to macOS and a tab elsewhere.
    g.click(text=say(g, "settings-tab-actions"), role="radiobutton" if g.os == "macos" else "tab", app=APP, timeout=15)
    g.click(text=say(g, "settings-write-action"), role="button", app=APP, timeout=10)
    g.wait_for(text=say(g, "settings-action-accepts"), app=APP, timeout=10)
    g.click(text=say(g, "settings-action-name"), role="textfield", app=APP, timeout=10)
    typed(g, "Name", name)
    g.check("Tab reaches the Text tick", tab_to(g, say(g, "settings-action-accepts-text"), "checkbox"), detail=g.find(role="checkbox", app=APP))
    toggle(g, say(g, "settings-action-accepts-text"), ticked=False)
    g.check("Tab reaches the Pictures tick", tab_to(g, say(g, "settings-action-accepts-image"), "checkbox"), detail=g.find(role="checkbox", app=APP))
    toggle(g, say(g, "settings-action-accepts-image"), ticked=True)
    g.check("Tab reaches the Prompt", tab_to(g, say(g, "settings-action-prompt")), detail=g.find(text="Prompt", app=APP))
    typed(g, "Prompt", prompt + " {{selection}}")
    g.check("Tab reaches Save Action", tab_to(g, say(g, "settings-save-action"), "button"), detail=g.find(text="Save Action", app=APP))
    g.press("enter")
    listed = g.wait_for(text=say(g, "settings-action-yours"), app=APP, timeout=10)
    g.check("the saved Action is listed as the user's own", listed["met"], detail=listed)
    g.screenshot("saved")
    # Restarted rather than Settings closed with Escape: an open Settings lists
    # the Action too, and would answer for the Palette both ways. The restart
    # also shows the Action outlives it.
    g.quit()
    g.launch()

    if g.os == "windows":
        listing = g.exec(["powershell", "-NoProfile", "-Command",
                          r"Get-ChildItem $env:APPDATA\demysto\actions | ForEach-Object { $_.Name }"]).stdout
    else:
        listing = g.exec(["ls", g.exec(["sh", "-c", f"echo {config_file(g, 'actions')}"]).stdout.strip()]).stdout
    files = [f for f in listing.split() if f.endswith(".toml")]
    texts = {f: g.get(config_file(g, "actions", f)) for f in files}
    mine = [t for t in texts.values() if prompt in t]
    g.check("the Action file says it runs on pictures only",
            len(mine) == 1 and 'accepts = ["image"]' in mine[0], detail=texts)

    copy_picture(g, 400, 300)
    staged = g.stage_text("nothing selected here " + nonce())
    g.press("right")
    g.press(palette_hotkey(g))
    # The Palette itself, before anything is read from it or Escape is pressed:
    # Escape pressed before it is up leaves it open over the editor, holding
    # the keyboard the next Staged document needs.
    picture = say(g, "palette-picture").split("{")[0].strip()
    shown = g.wait_for(text=picture, app=APP, timeout=15)
    g.check("the Palette opens on the picture", shown["met"], detail=shown)
    offered = g.find(text=name, app=APP)["matches"]
    g.check("the Palette offers the Action for a picture", offered != [], detail=offered)
    g.press("escape")
    closed = g.wait_for(text=picture, app=APP, gone=True, timeout=10)
    g.check("Escape closes the Palette on the picture", closed["met"], detail=closed)
    g.close_staged(staged)

    text = "a sign that reads STOP " + nonce()
    staged = g.stage_text(text, then=palette_hotkey(g))
    captured = g.wait_for(text=text, app=APP, timeout=15)
    if g.check("the Palette shows the selected text", captured["met"], detail=[captured, staged]):
        offered = g.find(text=name, app=APP)["matches"]
        g.check("the Palette leaves the Action out for text", not offered, detail=offered)
    else:
        g.skip("the Palette leaves the Action out for text", "the Palette did not take the selected text")
    g.press("escape")
    g.close_staged(staged)

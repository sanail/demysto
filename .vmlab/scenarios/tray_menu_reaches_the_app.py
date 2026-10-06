from _demysto import APP, configured_launch, language, mock, nonce, prompts, say, tray

LAUNCH = False


ITEMS = ["tray-open", "tray-chats", "tray-settings", "tray-quit"]


def scenario(g):
    answer = "Mock answer " + nonce()
    provider = mock(g, answer)
    configured_launch(g)
    window = g.wait_for(role="window", app=APP, timeout=5)
    g.check("a launch past the first run opens no window", not window["met"], detail=window)

    items = g.tray(tray(g))["items"]
    names = [item["name"] for item in items]
    for key in ITEMS:
        g.check(f"the tray menu has {say(g, key)}", say(g, key) in names, detail=names)
    g.check("the tray menu starts with the list of Actions, then the chats",
            names[:2] == [say(g, "tray-open"), say(g, "tray-chats")], detail=names)
    g.check("the tray menu has no submenu", not any(item["children"] for item in items), detail=items)

    g.tray(tray(g), choose=say(g, "tray-chats"))
    empty = g.wait_for(text=say(g, "result-no-chats"), app=APP, timeout=15)
    g.check("Chats… before any chat says there have been none", empty["met"], detail=empty)
    g.screenshot("no chats yet")
    g.press("escape")
    gone = g.wait_for(text=say(g, "result-no-chats"), app=APP, gone=True, timeout=10)
    g.check("Escape closes the empty chat window", gone["met"], detail=gone)

    text = "Ohm's law relates voltage and current " + nonce()
    staged = g.stage_text(text)
    g.tray(tray(g), choose=say(g, "tray-open"))
    palette = g.wait_for(text=text, app=APP, timeout=15)
    g.check("the first item opens the Palette on the foreground app's selection", palette["met"],
            detail=[palette, staged])
    g.screenshot("palette from the tray")

    # Named in the filter rather than taken from the highlight, which is not
    # the same Action on every OS.
    g.type(say(g, "action-explain-name"))
    g.press("enter")
    answered = g.wait_for(text=answer, app=APP, timeout=30)
    g.check("the Palette opened from the tray runs Explain", answered["met"], detail=[answered, provider.output()[-500:]])
    g.press("escape")
    closed = g.wait_for(text=answer, app=APP, gone=True, timeout=10)
    g.check("Escape closes the chat window", closed["met"], detail=closed)
    g.close_staged(staged)

    g.tray(tray(g), choose=say(g, "tray-chats"))
    back = g.wait_for(text=answer, app=APP, timeout=15)
    g.check("Chats… brings the chat window back on the chat it closed on", back["met"], detail=back)
    g.screenshot("chat reopened from the tray")
    # Typed without a click: the reopened window has to hold the focus.
    question = "And resistance? " + nonce()
    g.type(question)
    g.press("enter")
    asked = g.wait_for(text=question, app=APP, timeout=15)
    g.check("the reopened chat takes a follow-up", asked["met"] and question in prompts(provider),
            detail=[asked, prompts(provider)])
    g.press("escape")

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
            renamed == [say(g, key, other) for key in ITEMS], detail=renamed)

    g.tray(tray(g), choose=say(g, "tray-quit", other))
    gone = g.wait_for(process=APP, gone=True, timeout=15)
    g.check("Quit Demysto ends the app", gone["met"], detail=gone)

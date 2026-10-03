import re

from _demysto import APP, base_url, config_file, mock, tray


def fill(g, label, text):
    g.click(role="textfield", text=label, app=APP, timeout=10)
    g.type(text)


def scenario(g):
    mock(g, "Hi")

    welcome = g.wait_for(text="Welcome to Demysto", app=APP, timeout=20)
    g.check("a first run opens the welcome window", welcome["met"], detail=welcome)
    language = g.wait_for(text="Demysto found your language", app=APP, timeout=10)
    # The combobox shows its choice as its value, or, on Linux, in its name.
    boxes = g.find(role="combobox", app=APP)["matches"]
    g.check("the welcome flow starts in the OS's language", language["met"]
            and any("English" in (m["value"] or m["name"] or "") for m in boxes), detail=boxes)
    g.click(role="button", text="Continue", app=APP)

    asked = g.wait_for(text="Where the answers come from", app=APP, timeout=10)
    g.check("the welcome flow asks where answers come from", asked["met"], detail=asked)
    fill(g, "Name", "mock")
    fill(g, "Base URL", base_url(g))
    fill(g, "API key", "not-a-key")
    g.click(role="button", text="Fetch", app=APP)
    g.wait_for(text="mock-small", role="combobox", app=APP, timeout=15)
    g.click(role="button", text="Verify key", app=APP, timeout=10)
    verified = g.wait_for(text="mock-small answered.", app=APP, timeout=15)
    g.check("the welcome flow verifies the key against the Provider", verified["met"], detail=verified)
    g.screenshot("provider verified")

    # Accessibility (macOS only) and Startup lie between; each goes on by Continue.
    for _ in range(4):
        if g.find(text="That is everything", app=APP)["matches"]:
            break
        g.click(role="button", text="Continue", app=APP, timeout=10)
        g.wait_for(text="Step", app=APP, timeout=5)
    last = g.wait_for(text="That is everything", app=APP, timeout=10)
    g.check("the welcome flow reaches its last step", last["met"], detail=last)
    g.click(role="button", text="Start using Demysto", app=APP, timeout=10)
    ended = g.wait_for(text="Welcome to Demysto", app=APP, gone=True, timeout=10)
    g.check("the welcome flow ends", ended["met"], detail=ended)

    settings = g.get(config_file(g, "settings.toml"))
    g.check("the Settings File holds the Provider and its Model",
            base_url(g) in settings and "mock-small" in settings and "welcomed = true" in settings,
            detail=settings)
    g.check("confirming the language the OS gave writes no language",
            not re.search(r"(?m)^language\s*=", settings), detail=settings)

    g.tray(tray(g), choose="Settings…", timeout=10)
    shown = g.wait_for(text="mock/mock-small", app=APP, timeout=15)
    g.check("Settings, opened from the tray, shows the Provider's Model without a restart",
            shown["met"], detail=shown)
    g.screenshot("settings")

    g.quit()
    g.launch()
    again = g.wait_for(text="Welcome to Demysto", app=APP, timeout=5)
    g.check("a second launch does not open the welcome window", not again["met"], detail=again)
    g.tray(tray(g), choose="Settings…", timeout=10)
    kept = g.wait_for(text="mock/mock-small", app=APP, timeout=15)
    g.check("Settings, opened from the tray, shows the Provider's Model after a restart",
            kept["met"], detail=kept)

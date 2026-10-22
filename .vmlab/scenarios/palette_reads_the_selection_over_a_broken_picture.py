from _demysto import APP, configured_launch, copy_a_broken_picture, nonce, palette_hotkey, say

LAUNCH = False
# The selection is read with a synthetic copy, which Wayland refuses.
SESSIONS = ["x11"]
# What this measures is the same in every Lab language.
LANGUAGES = ["en-US"]


def scenario(g):
    configured_launch(g)
    try:
        copy_a_broken_picture(g)
        text = "Ohm's law relates voltage and current " + nonce()

        staged = g.stage_text(text, then=palette_hotkey(g))
        g.check("the Hotkey went to the editor holding the selection",
                staged["frontmost"] == staged["app"] and staged["selected"] == text, detail=staged)
        palette = g.wait_for(text=text, app=APP, timeout=15)
        g.check("the Palette shows the selected text over a picture nobody can read", palette["met"], detail=palette)
        origin = g.wait_for(text=say(g, "palette-origin-selection"), app=APP, timeout=5)
        g.check("the Palette says the text is the Selection", origin["met"], detail=origin)
        g.screenshot("palette")

        g.press("escape")
        closed = g.wait_for(text=text, app=APP, gone=True, timeout=10)
        g.check("Escape closes the Palette", closed["met"], detail=closed)
        g.close_staged(staged)
    finally:
        # What broke the clipboard outlives this Scenario on Windows.
        g.set_clipboard("")

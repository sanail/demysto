from _demysto import APP, configured_launch, nonce, palette_hotkey, say

LAUNCH = False
# What this measures is the same in every Lab language.
LANGUAGES = ["en-US"]


def scenario(g):
    configured_launch(g)
    clip = "clipboard text " + nonce()
    g.set_clipboard(clip)
    text = "Ohm's law relates voltage and current " + nonce()

    staged = g.stage_text(text, then=palette_hotkey(g))
    g.check("the Hotkey went to the editor holding the selection",
            staged["frontmost"] == staged["app"] and staged["selected"] == text, detail=staged)
    palette = g.wait_for(text=text, app=APP, timeout=15)
    g.check("the Palette shows the selected text", palette["met"], detail=palette)
    origin = g.wait_for(text=say(g, "palette-origin-selection"), app=APP, timeout=5)
    g.check("the Palette says the text is the Selection", origin["met"], detail=origin)
    g.check("the Palette does not show the clipboard instead",
            not g.find(text=clip, app=APP)["matches"], detail=g.find(text=clip, app=APP))
    g.screenshot("palette")

    g.press("escape")
    closed = g.wait_for(text=text, app=APP, gone=True, timeout=10)
    g.check("Escape closes the Palette", closed["met"], detail=closed)
    back = g.clipboard()["text"]
    g.check("Capture gives the clipboard back", back == clip, detail=back)
    g.close_staged(staged)

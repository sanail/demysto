from _demysto import APP, configured_launch, nonce, palette_hotkey, say

LAUNCH = False
# What this measures is the same in every Lab language.
LANGUAGES = ["en-US"]


def scenario(g):
    configured_launch(g)
    clip = "copied before " + nonce()
    g.set_clipboard(clip)

    # An editor in front with its text unselected: staged, then the selection
    # collapsed to the cursor.
    staged = g.stage_text("nothing selected here " + nonce())
    g.press("right")
    g.press(palette_hotkey(g))
    g.check("the Hotkey went to an editor with nothing selected",
            staged["frontmost"] == staged["app"], detail=staged)
    palette = g.wait_for(text=clip, app=APP, timeout=15)
    g.check("the Palette shows the clipboard's text", palette["met"], detail=palette)
    origin = g.wait_for(text=say(g, "palette-origin-clipboard"), app=APP, timeout=5)
    g.check("the Palette says the text came from the clipboard", origin["met"], detail=origin)
    g.screenshot("palette on the clipboard")
    g.press("escape")
    closed = g.wait_for(text=clip, app=APP, gone=True, timeout=10)
    g.check("Escape closes the Palette", closed["met"], detail=closed)

    g.set_clipboard("")
    g.focus(staged["app"])
    g.press(palette_hotkey(g))
    empty = g.wait_for(text=say(g, "palette-nothing-captured"), app=APP, timeout=15)
    g.check("with an empty clipboard the Palette says nothing is selected", empty["met"], detail=empty)
    g.press("escape")
    g.close_staged(staged)

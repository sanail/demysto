from _demysto import APP, in_app, configured_launch, copy_picture, mock, nonce, palette_hotkey, pictures, say

LAUNCH = False

# Larger than the 1568-pixel ceiling, so the fitted picture and the original differ.
WIDTH, HEIGHT = 2000, 1200
FITTED = (1568, 941)


def scenario(g):
    answer = "Mock description " + nonce()
    provider = mock(g, answer)
    configured_launch(g)
    copy_picture(g, WIDTH, HEIGHT)

    # An editor in front with nothing selected: the Capture falls back on the clipboard.
    staged = g.stage_text("nothing selected here " + nonce())
    g.press("right")
    g.press(palette_hotkey(g))
    shown = g.wait_for(text=say(g, "palette-picture", dimensions=f"{WIDTH} × {HEIGHT}"), app=APP, timeout=15)
    g.check("the Palette shows the copied picture with its size", shown["met"], detail=shown)
    offered = in_app(g, say(g, "action-describe-image-name"))
    g.check("the Palette offers Describe image for a picture", offered != [], detail=offered)
    explain = in_app(g, say(g, "action-explain-name"))
    g.check("the Palette leaves out Explain, which runs on text only", not explain, detail=explain)
    g.screenshot("palette on a picture")

    # Named in the filter rather than taken from the highlight, which is not
    # the same Action on every OS.
    g.type(say(g, "action-describe-image-name"))
    g.press("enter")
    answered = g.wait_for(text=answer, app=APP, timeout=30)
    g.check("Describe image shows the vision Model's answer", answered["met"], detail=[answered, provider.output()[-500:]])
    sent = pictures(provider)
    g.check("the Model is sent the picture fitted under the ceiling", sent == [FITTED], detail=sent)
    g.screenshot("answer about the picture")
    g.press("escape")
    g.close_staged(staged)

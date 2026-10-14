from _demysto import APP, configured_launch, mock, nonce, palette_hotkey, prompts, say, shows_answer

LAUNCH = False
# The selection is read with a synthetic copy, which Wayland refuses.
SESSIONS = ["x11"]
# What this measures is the same in every Lab language.
LANGUAGES = ["en-US"]


def scenario(g):
    tag = nonce()
    answer = "Mock answer " + tag
    provider = mock(g, answer)
    configured_launch(g)

    # Cyrillic, so the Selection's way through Capture, the Palette and the
    # request is checked for more than ASCII.
    text = "Закон Ома связывает силу тока и напряжение " + tag
    staged = g.stage_text(text, then=palette_hotkey(g))
    palette = g.wait_for(text=text, app=APP, timeout=15)
    g.check("the Palette shows a Cyrillic selection", palette["met"], detail=[palette, staged])
    listed = g.find(text=say(g, "action-explain-name"), app=APP)["matches"]
    g.check("the Palette lists Explain", listed != [], detail=listed)

    # Named in the filter rather than taken from the highlight, which is not
    # the same Action on every OS.
    g.type(say(g, "action-explain-name"))
    g.press("enter")
    answered, seen = shows_answer(g, answer)
    g.check("Enter runs Explain and the Conversation shows the Model's answer",
            answered, detail=[seen, provider.output()])
    g.screenshot("answer")
    sent = prompts(provider)
    g.check("the prompt carries the selection intact", any(text in p for p in sent), detail=sent)

    g.press("escape")
    g.close_staged(staged)

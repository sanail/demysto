from _demysto import APP, configured_launch, mock, nonce, palette_hotkey, prompts

LAUNCH = False


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
    listed = g.wait_for(text="Explain", app=APP, timeout=5)
    g.check("the Palette lists Explain", listed["met"], detail=listed)

    g.press("enter")
    answered = g.wait_for(text=answer, app=APP, timeout=30)
    g.check("Enter runs Explain and the Conversation shows the Model's answer",
            answered["met"], detail=[answered, provider.output()])
    g.screenshot("answer")
    sent = prompts(provider)
    g.check("the prompt carries the selection intact", any(text in p for p in sent), detail=sent)

    g.press("escape")
    g.close_staged(staged)

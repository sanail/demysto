from _demysto import APP, configured_launch, explain_hotkey, mock, nonce, say

LAUNCH = False


def scenario(g):
    tag = nonce()
    # Slow enough to put the window away while the Run is still in flight.
    mock(g, "refused " + tag, status=500, delay=6)
    stated, chord = explain_hotkey(g)
    configured_launch(g, {"explain": f'hotkey = "{stated}"\n'})

    text = "unseen failure " + tag
    staged = g.stage_text(text, then=chord)
    opened = g.wait_for(text=text, app=APP, timeout=15)
    g.check("the Action's Hotkey opens the Conversation window", opened["met"], detail=[opened, staged])
    g.press("escape")
    away = g.wait_for(text=text, app=APP, gone=True, timeout=10)
    g.check("Escape puts the Conversation window away", away["met"], detail=away)
    told = g.wait_for(notification=tag, timeout=30)
    g.check("a failure nobody can see is notified, with the Provider's error", told["met"], detail=told)
    titles = [n["title"] for n in told.get("notifications", [])]
    g.check("the notification says Demysto could not answer",
            say(g, "notification-could-not-answer") in titles, detail=told.get("notifications"))
    g.close_staged(staged)

    seen = "seen failure " + tag
    staged = g.stage_text(seen, then=chord)
    shown = g.wait_for(text="refused " + tag, app=APP, timeout=30)
    g.check("a failure in a visible Conversation shows there", shown["met"], detail=shown)
    g.screenshot("failure in the conversation")
    since = g.wait_for(notification="refused " + tag, timeout=8)
    g.check("a failure shown in the Conversation is not also notified",
            len(since.get("notifications", [])) == 1, detail=since)
    g.press("escape")
    g.close_staged(staged)

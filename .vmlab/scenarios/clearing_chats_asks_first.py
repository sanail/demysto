from _demysto import APP, CATALOGUE_LANGUAGES, configure, approve_shortcuts, largest_window, mock, nonce, palette_hotkey, say, tray

LAUNCH = False
TIMEOUT = 1200
# The selection is read with a synthetic copy, which Wayland refuses.
SESSIONS = ["x11"]
# Every catalogue is switched to inside one Guest; the system's language changes nothing here.
LANGUAGES = ["en-US"]

SLACK = 1  # a logical pixel of rounding between how the toolkit and the window report bounds


def largest(g, window, **where):
    """The largest node matching where that lies inside window, so that a hidden Palette is left out."""
    found = [b for b in (m.get("bounds") for m in g.find(app=APP, **where)["matches"]) if b
             and b["x"] >= window["x"] and b["y"] >= window["y"] and b["y"] + b["h"] <= window["y"] + window["h"]]
    return max(found, key=lambda b: b["w"] * b["h"]) if found else None


def apart(a, b, slack):
    return bool(a and b) and (min(a["x"] + a["w"], b["x"] + b["w"]) - max(a["x"], b["x"]) <= slack
                              or min(a["y"] + a["h"], b["y"] + b["h"]) - max(a["y"], b["y"]) <= slack)


def taken_back(g, how, cancel, answer):
    """Checks the question is gone and the Conversation it was about is still there."""
    gone = g.wait_for(role="button", text=cancel, app=APP, gone=True, timeout=10)
    g.check(f"{how} takes the question back and keeps the chat",
            gone["met"] and g.find(text=answer, app=APP)["matches"] != [], detail=gone)


def scenario(g):
    answer = "Mock answer " + nonce()
    mock(g, answer)
    configure(g)
    slack = SLACK * g.displays()["displays"][0]["scale"]

    for lang in CATALOGUE_LANGUAGES:
        g.quit()
        g.launch(env={"DEMYSTO_LANGUAGE": lang})
        approve_shortcuts(g)

        text = "Ohm's law relates voltage and current " + nonce()
        staged = g.stage_text(text, then=palette_hotkey(g))
        palette = g.wait_for(text=text, app=APP, timeout=15)
        if not g.check(f"{lang}: the Palette opens on the selection", palette["met"], detail=[palette, staged]):
            g.close_staged(staged)
            continue
        explain = say(g, "action-explain-name", lang)
        g.type(explain)
        g.press("enter")
        shown = g.wait_for(text=answer, app=APP, timeout=30)
        if not g.check(f"{lang}: Explain answers in a chat", shown["met"], detail=shown):
            g.close_staged(staged)
            continue

        clear = say(g, "result-clear", lang)
        asked = say(g, "result-clear-asked", lang)
        confirm = say(g, "result-clear-confirm", lang)
        cancel = say(g, "result-clear-cancel", lang)
        g.click(role="button", text=clear, app=APP, timeout=10)
        question = g.wait_for(text=asked, app=APP, timeout=10)
        g.check(f"{lang}: × asks before clearing", question["met"], detail=question)
        g.screenshot(f"{lang} asked")

        window = largest_window(g.tree(app=APP))
        parts = {
            # The largest node by that name: the group the question labels, which holds its buttons.
            "question": largest(g, window, text=asked),
            "confirm": largest(g, window, role="button", text=confirm),
            "cancel": largest(g, window, role="button", text=cancel),
            "action": largest(g, window, text=explain),
        }
        missing = [name for name, b in parts.items() if not b]
        outside = {name: b for name, b in parts.items() if b and (
            b["x"] < window["x"] - slack or b["x"] + b["w"] > window["x"] + window["w"] + slack)}
        g.check(f"{lang}: the question fits in the window", not missing and not outside,
                detail=[missing, window, parts])
        g.check(f"{lang}: the question stays clear of the Action's name",
                apart(parts["question"], parts["action"], slack), detail=parts)

        g.press("escape")
        taken_back(g, f"{lang}: Escape", cancel, answer)
        g.click(role="button", text=clear, app=APP, timeout=10)
        g.click(role="button", text=cancel, app=APP, timeout=10)
        taken_back(g, f"{lang}: Cancel", cancel, answer)
        # Enter reaches × only if the focus went back to it.
        g.press("enter")
        again = g.wait_for(role="button", text=cancel, app=APP, timeout=10)
        g.check(f"{lang}: after Cancel the focus is back on ×", again["met"], detail=again)
        g.press("escape")
        taken_back(g, f"{lang}: Escape from a question reopened by keyboard", cancel, answer)

        g.click(role="button", text=clear, app=APP, timeout=10)
        g.click(role="button", text=confirm, app=APP, timeout=10)
        gone = g.wait_for(text=answer, app=APP, gone=True, timeout=10)
        g.check(f"{lang}: clearing closes the chat window", gone["met"], detail=gone)

        g.tray(tray(g), choose=say(g, "tray-chats", lang), timeout=15)
        empty = g.wait_for(text=say(g, "result-no-chats", lang), app=APP, timeout=15)
        g.check(f"{lang}: after clearing, Chats… says there are none",
                empty["met"] and g.find(text=answer, app=APP)["matches"] == [], detail=empty)
        g.press("escape")
        g.close_staged(staged)

import re

from _demysto import APP, HERE, nonce, say

LAUNCH = False  # no Settings File: a first run, which opens the welcome window
# Only Windows lets a window created hidden take the foreground, in any language.
OS = ["windows"]
LANGUAGES = ["en-US"]


def scenario(g):
    staged = g.stage_text("typing here " + nonce())
    g.press("right")
    script = g.put(r"%TEMP%\foreground.ps1", (HERE / "mock" / "foreground.ps1").read_text())
    tracer = g.spawn(["powershell", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", script, "20"])
    g.wait_for(log=tracer.log, pattern="tracing", timeout=20)
    g.launch()
    welcome = g.wait_for(text=say(g, "welcome-title"), app=APP, timeout=30)
    g.check("the welcome window opens", welcome["met"], detail=welcome)
    g.wait_for(log=tracer.log, pattern="done", timeout=40)
    trace = tracer.output()
    hidden = [line for line in trace.splitlines() if re.match(r"\d+ demysto visible=False", line)]
    g.check("no hidden Demysto window takes the foreground while it starts", not hidden, detail=trace)
    g.close_staged(staged)

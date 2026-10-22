from _demysto import (APP, check_fitted, check_no_resize, configured_launch, nonce, nothing_selected, palette_hotkey,
                      palette_layout, say, watch_the_palette)

LAUNCH = False

TCC = "/Library/Application Support/com.apple.TCC/TCC.db"
ACCESSIBILITY = ("client = 'app.demysto' AND service IN "
                 "('kTCCServiceAccessibility', 'kTCCServicePostEvent')")


def break_the_capture(g):
    """Puts the Guest where a Capture fails the way it does for users there,
    and answers the start of what the Palette then says."""
    if g.os == "macos":
        # The permission the Lab grants, taken back: the failure only macOS has.
        g.exec(["sudo", "-n", "sqlite3", TCC, f"UPDATE access SET auth_value = 0 WHERE {ACCESSIBILITY}"])
        return say(g, "capture-no-accessibility")
    if g.os == "windows":
        # A picture that is not one, under the format browsers copy pictures in.
        script = g.put(r"%TEMP%\demysto-bad-picture.ps1", (
            "Add-Type -AssemblyName System.Windows.Forms\n"
            "$data = New-Object System.Windows.Forms.DataObject\n"
            "$data.SetData('PNG', (New-Object System.IO.MemoryStream(,[byte[]](1..64))))\n"
            "[System.Windows.Forms.Clipboard]::SetDataObject($data, $true)\n"))
        g.exec(["powershell", "-STA", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", script])
    else:
        # Text that is not UTF-8 under the type that promises it; on Wayland it
        # reaches Demysto through Xwayland.
        path = g.put("/tmp/demysto-bad-text", b"\xff\xfe not text " + nonce().encode())
        g.spawn(["xclip", "-quiet", "-selection", "clipboard", "-t", "UTF8_STRING", "-i", path])
        g.wait_for(exec=["sh", "-c", "xclip -selection clipboard -t TARGETS -o | grep -q UTF8_STRING"], timeout=10)
    return say(g, "capture-clipboard-unavailable").split("{ $")[0].strip()


def scenario(g):
    configured_launch(g)
    try:
        said = break_the_capture(g)
        staged = nothing_selected(g)
        watching = watch_the_palette(g)
        g.press(palette_hotkey(g))
        failed = g.wait_for(text=said, app=APP, timeout=15)
        g.check("the Palette says the Capture failed", failed["met"], detail=failed)
        check_no_resize(g, watching, "on a failed Capture,")
        window, footer, _ = palette_layout(g, say(g, "palette-keys-closing"))
        check_fitted(g, window, footer, "on a failed Capture")
        g.screenshot("failed capture")
        g.press("escape")
        g.wait_for(text=said, app=APP, gone=True, timeout=10)
        if staged:
            g.close_staged(staged)
    finally:
        # What broke the clipboard outlives this Scenario on Windows.
        g.set_clipboard("")
        if g.os == "macos":
            g.exec(["sudo", "-n", "sqlite3", TCC, f"UPDATE access SET auth_value = 2 WHERE {ACCESSIBILITY}"])

# Prints the bounds of Demysto's windows on screen each time they change, or
# "none" when there are none: fast enough to see a resize no user should.
# Wayland shows no client another's windows, so they are asked of the GNOME
# Shell extension every vmlab Guest on Wayland carries.
import json
import subprocess

from gi.repository import Gio

bus = Gio.bus_get_sync(Gio.BusType.SESSION)
# Asked once: the app is running before anything is watched.
pids = {int(pid) for pid in subprocess.run(["pgrep", "-x", "demysto"], capture_output=True, text=True).stdout.split()}
last = None
while True:
    found = json.loads(bus.call_sync("org.vmlab.Shell", "/org/vmlab/Shell", "org.vmlab.Shell", "Windows",
                                     None, None, Gio.DBusCallFlags.NONE, -1, None).unpack()[0])
    now = " | ".join(" ".join(str(n) for n in window["frame"]) for window in found["windows"]
                     if window["pid"] in pids and window["title"] == "Demysto" and not window["hidden"]) or "none"
    if now != last:
        print(now, flush=True)
        last = now

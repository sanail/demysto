# Puts the one monitor at WIDTHxHEIGHT without saving it, or with "current"
# prints the size it is at.
import sys

from gi.repository import Gio, GLib

bus = Gio.bus_get_sync(Gio.BusType.SESSION)


def call(method, args=None):
    return bus.call_sync("org.gnome.Mutter.DisplayConfig", "/org/gnome/Mutter/DisplayConfig",
                         "org.gnome.Mutter.DisplayConfig", method, args, None, Gio.DBusCallFlags.NONE, -1,
                         None).unpack()


serial, monitors, _, _ = call("GetCurrentState")
(connector, _, _, _), modes, _ = monitors[0]
if sys.argv[1] == "current":
    current = next(mode for mode in modes if mode[6].get("is-current"))
    print(f"{current[1]}x{current[2]}")
    sys.exit()
width, height = map(int, sys.argv[1].split("x"))
mode = next(mode for mode in modes if (mode[1], mode[2]) == (width, height))
# 1 is a temporary configuration, which GNOME neither saves nor asks about.
call("ApplyMonitorsConfig", GLib.Variant("(uua(iiduba(ssa{sv}))a{sv})",
                                         (serial, 1, [(0, 0, 1.0, 0, True, [(connector, mode[0], {})])], {})))

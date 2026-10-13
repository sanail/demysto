"""Shows the picture at argv[1] full screen, stretched to it, until killed:
blocks.py PATH TITLE"""
import sys

import gi

gi.require_version("Gtk", "3.0")
gi.require_version("Gdk", "3.0")
from gi.repository import Gdk, GdkPixbuf, Gtk  # noqa: E402

path, title = sys.argv[1], sys.argv[2]
# In the monitor's logical pixels, which GTK scales up itself.
display = Gdk.Display.get_default()
area = (display.get_primary_monitor() or display.get_monitor(0)).get_geometry()
win = Gtk.Window(title=title)
win.add(Gtk.Image.new_from_pixbuf(GdkPixbuf.Pixbuf.new_from_file_at_scale(path, area.width, area.height, False)))
win.connect("destroy", Gtk.main_quit)
win.fullscreen()
win.show_all()
Gtk.main()

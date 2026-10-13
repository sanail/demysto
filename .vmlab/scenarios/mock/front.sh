#!/bin/sh
# Prints the process whose window is in the foreground, by its executable:
# ps cuts names at 15 characters.
basename "$(readlink "/proc/$(xdotool getactivewindow getwindowpid)/exe")"

#!/bin/sh
# Prints the bounds of Demysto's window on screen each time they change, or
# "none" when it is not: fast enough to see a resize no user should.
last=
while :; do
  now=$(xwininfo -name Demysto 2>/dev/null | awk '
    /Absolute upper-left X/ {x=$NF} /Absolute upper-left Y/ {y=$NF}
    /Width:/ {w=$NF} /Height:/ {h=$NF} /Map State/ {s=$NF}
    END {print (s == "IsViewable" ? x " " y " " w " " h : "none")}')
  if [ "$now" != "$last" ]; then
    echo "$now"
    last=$now
  fi
done

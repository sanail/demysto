#!/usr/bin/env bash
# Refuse an AppImage that carries the host's Wayland client library.
#
# `prepare-appimage.sh` keeps it out by handing the bundler an output plugin of
# our own. If the bundler ever stops looking for that plugin where the script
# leaves it, nothing fails and nothing is logged — the library comes back, and
# the first anybody hears of it is a user reporting four white rectangles. So
# the built artifact is asked directly. See ADR-0016.
set -euo pipefail

image="${1:?usage: check-appimage.sh <path to .AppImage>}"
[ -f "$image" ] || { echo "no such AppImage: $image" >&2; exit 1; }
# Absolute, because the extraction below happens in a directory of its own.
image="$(cd "$(dirname "$image")" && pwd)/$(basename "$image")"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

# `--appimage-extract` is the runtime's own and needs no FUSE; the pattern keeps
# a quarter of a gigabyte from being written out to answer one question.
extract() { (cd "$work" && "$image" --appimage-extract "$1" >/dev/null); }

# A runtime that took no notice of the pattern and unpacked nothing would say
# the same thing about the library as one that unpacked everything and found it
# absent. So something known to be there is asked for first.
extract 'AppRun'
if [ ! -e "$work/squashfs-root/AppRun" ]; then
  echo "$image: --appimage-extract produced no AppRun, so nothing below is evidence" >&2
  exit 1
fi

# Both depths the shim clears. Tauri's AppDir keeps some libraries under the
# architecture's own directory, and a copy that came back one level down is
# exactly the regression this is here to catch.
for where in 'usr/lib*' 'usr/lib*/*'; do
  extract "$where/libwayland-client.so*"
  if compgen -G "$work/squashfs-root/$where/libwayland-client.so*" >/dev/null; then
    echo "$image bundles libwayland-client.so, which the host's Mesa needs its own copy of" >&2
    echo "see ADR-0016 and scripts/prepare-appimage.sh" >&2
    exit 1
  fi
done

echo "$(basename "$image") leaves libwayland-client.so to the host"

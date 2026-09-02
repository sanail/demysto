#!/usr/bin/env bash
# Put a shim where the Tauri bundler looks for linuxdeploy's AppImage output
# plugin, so that the AppImage is packed without the host's own Wayland client
# library. Run before `tauri build` on Linux; see ADR-0016.
#
# An AppImage puts its own `usr/lib` first on `LD_LIBRARY_PATH`, and everything
# the process loads later inherits it — including the host's Mesa, which the
# host's libEGL opens by hand. A `libwayland-client.so.0` from the build image
# then answers Mesa's imports, and a Mesa newer than the build image asks for
# symbols that copy does not have. EGL refuses to start, WebKit's renderer
# aborts, and every window is a white rectangle.
set -euo pipefail

arch="$(uname -m)"
cache="${XDG_CACHE_HOME:-$HOME/.cache}/tauri"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# The bundler's own source for the plugin, so that what runs is the build it
# would have downloaded itself.
upstream="https://github.com/linuxdeploy/linuxdeploy-plugin-appimage/releases/download/continuous/linuxdeploy-plugin-appimage-$arch.AppImage"

mkdir -p "$cache"
# Named so that it is not itself taken for a plugin: linuxdeploy reads
# `linuxdeploy-plugin-<name>` out of every directory on PATH, and this one sits
# next to the shim.
real="$cache/appimage-plugin-upstream.AppImage"
if [ ! -f "$real" ]; then
  # Downloaded beside the name and moved onto it: `curl -f` still truncates its
  # output file on an HTTP error, and the empty file left behind would be taken
  # for a download that had succeeded on every later run.
  curl -fsSL -o "$real.part" "$upstream"
  mv "$real.part" "$real"
fi
chmod +x "$real"

cc -O2 -o "$cache/linuxdeploy-plugin-appimage.AppImage" \
   -DPLUGIN="\"$real\"" "$here/appimage-shim.c"

echo "the AppImage output plugin in $cache now leaves libwayland-client.so to the host"

# ADR-0016: The AppImage leaves the host's Wayland client library to the host

Status: accepted

## Context

The `.AppImage` built by ticket 16 draws nothing. Every window is a white
rectangle, on a desktop where the same build as a bare binary draws correctly,
minutes apart. The renderer says why once per window and then stops:

```
Could not create default EGL display: EGL_BAD_PARAMETER. Aborting...
```

The cause is not WebKit and not the bundled WebKit's age. An AppImage puts its
own `usr/lib` first on `LD_LIBRARY_PATH`, and everything the process loads
afterwards inherits that — including the host's Mesa, which the host's libEGL
opens by hand at the first EGL call. The bundle carries a
`libwayland-client.so.0` from the Ubuntu 22.04 build image, that copy answers
Mesa's imports, and a Mesa newer than the build image asks it for three symbols
it does not have: `wl_fixes_interface`, `wl_display_create_queue_with_name`,
`wl_display_dispatch_queue_timeout`. Mesa fails to initialise, EGL returns
`EGL_BAD_PARAMETER`, and WebKit's renderer process — one process behind all four
windows, which is why all four are affected — aborts.

Measured rather than reasoned: on the Linux stand, `eglinfo` succeeds and
`LD_LIBRARY_PATH=<AppDir>/usr/lib eglinfo` fails, and putting back one bundled
library at a time names `libwayland-client.so.0` and nothing else.

None of the environment variables ADR-0014 lists changes any of it, because
none of them is about this. Nor does building on a newer runner: those three
symbols arrived in wayland 1.23, which Ubuntu 24.04 does not carry either, so
the newer image would move the glibc floor for nothing. The fault is structural
— an AppImage that ships any part of the graphics stack breaks against a host
newer than itself, whenever that is — and `libwayland-client.so.0` is on the
AppImage excludelist for exactly this reason.

## Decision

The AppImage does not ship `libwayland-client.so.0`; the host's copy is used.
`scripts/prepare-appimage.sh` runs before `tauri build` and puts
`scripts/appimage-shim.c`, compiled, where the bundler looks for linuxdeploy's
AppImage output plugin. The shim removes the excluded libraries from the AppDir
and hands the real plugin the same arguments.

The output plugin is the hook because it is the last thing to touch the AppDir
before it is packed. linuxdeploy has `--exclude-library` and the bundler passes
it no arguments of ours; and even given them, linuxdeploy's GTK plugin runs
afterwards, calls linuxdeploy again itself, and puts the library back — GTK's
Wayland input modules depend on it. The shim is a compiled program because the
bundler zeroes three bytes at offset 8 of every tool it runs, the AppImage magic
it removes so the tool runs without FUSE; in an ELF those bytes are padding, and
in a shell script they are the middle of the shebang line.

The arrangement is quiet when it breaks — if the bundler ever stops reading that
plugin from that directory, nothing fails and nothing is logged, and the library
comes back. So `scripts/check-appimage.sh` asks the built artifact directly, and
the release job runs it.

The glibc floor ADR-0004 chose is unchanged: the Linux artifacts are still built
on Ubuntu 22.04.

## Consequences

The AppImage now needs `libwayland-client.so.0` from the host, which is what the
excludelist asks of every AppImage and what the `.deb` has always done. Any
desktop that can run a GTK or Qt application has it; a machine that does not
would meet a loader error naming the library instead of a window that draws
nothing, which is the better of the two failures.

Only that one library is dropped. It is the only entry on the current
excludelist that the bundle contained, and the list linuxdeploy carries covers
the rest — `libGL`, `libEGL`, `libdrm`, `libgbm`, `libxcb` and the others were
never bundled.

Verified on the Linux stand against Kubuntu 26.04 aarch64 with no graphics
acceleration — the desktop the fault was found on — where the Palette and the
Conversation window now draw from the `.AppImage`.

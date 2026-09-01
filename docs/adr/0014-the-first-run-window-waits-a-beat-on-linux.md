# ADR-0014: The first-run window waits a beat on Linux rather than changing WebKit's environment

Status: accepted

## Context

The first-run window is the one window Demysto opens by itself, in the first
instants of a launch. On a Linux desktop with no graphics acceleration — a
virtual machine whose Mesa answers `Accelerated: no` — a window shown there does
not paint. It comes up as a correct, complete, white rectangle: the page is
loaded, the accessibility tree reads every line of it back, the elements report
their places on screen, and not one pixel is drawn. A resize does not bring it
back, and nothing is written to any log.

The three windows Demysto has always had never meet this, and that is the same
fact rather than luck: nothing shows any of them until the user asks for one,
which is never in the first instants. Shown a beat later, the same window draws;
the threshold measured on that desktop is under a fifth of a second.

Tauri documents a ladder of environment variables for blank windows on Linux —
`__NV_DISABLE_EXPLICIT_SYNC`, then `WEBKIT_DISABLE_DMABUF_RENDERER`, then
`WEBKIT_DISABLE_COMPOSITING_MODE` — and the second of them does fix this
outright.

## Decision

Demysto sets none of them. `welcome::reveal` waits a second and then shows the
window from the main thread, and that waiting is compiled for Linux alone:
WKWebView and WebView2 draw a window shown in the first instants as readily as
one shown an hour in, watched on both, so there is nothing there for a wait to
fix.

The variables were rejected because every machine with a working graphics card
would pay for them, in a slower path it never needed, for a fault it does not
have. Tauri's own guidance is the same: ship an unconditional override only for
an application verified to be affected. Detecting the affected machines instead
— reading the driver behind the render node — was rejected as a heuristic that
misfires in both directions, when the application can simply avoid the moment.

## Consequences

A fresh installation on Linux sees its first window a second after launch. It is
paid once in the life of an installation, by somebody who has just started an
application and is watching it come up; every later window is shown when the
user asks for one and waits for nothing.

The remedy is a race rather than a proof. The margin is five times the measured
threshold, and the failure it guards against is silent, so a desktop slower than
that would show the same blank window — the reason the measurement, not just the
remedy, is written down here and in `welcome.rs`.

Anybody who does hit it can still export `WEBKIT_DISABLE_DMABUF_RENDERER=1`
themselves: nothing in Demysto reads or overrides it.

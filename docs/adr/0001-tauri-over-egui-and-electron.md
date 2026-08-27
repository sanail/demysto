---
status: accepted
---

# Tauri 2 rather than a pure-Rust GUI or Electron

Demysto is a resident utility, so bundle size and idle memory are first-order
concerns, which rules Electron out (~150 MB bundle, ~300 MB idle). Among the
compact options we chose **Tauri 2 with a system webview** over egui, Slint and
Iced, because the screen the product actually lives in is a streaming Markdown
conversation — code blocks, syntax highlighting, text selection, copy — which a
webview renders for free and an immediate-mode Rust GUI does not.

## Considered options

**egui/eframe** — a single ~10 MB binary at ~40-80 MB idle, no system
dependencies, and it sidesteps Tauri's weakest platform (WebKitGTK on Linux).
Rejected because `egui_commonmark` gives weak cross-block text selection,
mediocre tables and lists, syntax highlighting only via `syntect`, and fonts
must be embedded — which would cost 10-16 MB the day a CJK interface language is
added. Re-laying out Markdown on every stream chunk is also manual work there.

**Slint** — the smallest footprint of the three, but has no Markdown renderer at
all. **Iced** — same font and Markdown caveats as egui, with a less mature
widget set. **Dioxus desktop** — the same system webview as Tauri, so not a
distinct option.

## Consequences

The choice does not affect the native mechanics: global hotkeys, tray, clipboard
and synthetic key injection come from standalone crates (`global-hotkey`,
`tray-icon`, `arboard`, `enigo`) that work identically under any Rust GUI.

The accepted risk is Linux: WebKitGTK is Tauri's weakest backend, it adds a
`webkit2gtk-4.1` system dependency, and transparency and always-on-top behave
inconsistently across compositors. If Linux becomes a first-class target rather
than a supported one, this decision is the first thing to revisit.

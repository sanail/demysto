# Changelog

What each release brought, newest first.

## 0.1.3 — 2026-09-06

A release about Settings: what it can say about the rest of the system, and what
it says to a screen reader.

### Starting at login

- Settings turns starting at login on and off, and shows what the login items
  hold rather than what Demysto last asked of them. The choice had been offered
  once, during the first run, with nowhere afterwards to change one's mind.
- The box answers in turn. A click into a window that did not have focus asks
  the system two questions at once, and the older answer could land last and
  undo the click. A list that could not be read is no longer reported as a list
  that says no.

### What Settings knows

- The Provider the first run wrote is in Settings straight away. The window had
  read the file before the flow finished writing it, and the first Provider
  appeared only after a restart.
- A Model is written as it was typed. macOS autocorrection was capitalising
  identifiers in the first-run and Settings fields, and `mock-small` reached the
  file as `Mock-small`.

### Reading the window aloud

- The Hotkey row in Settings says what it is for, what is bound now, that a
  recording is under way, and which combinations are allowed. On Linux the row
  had been two unlabelled buttons and nothing besides. The per-Action Hotkey row
  in the Action editor answers the same way.

## 0.1.2 — 2026-09-04

A release about the mouse, and three places where it could start something only
the keyboard could finish.

### The pointer

- Recording a Hotkey can be called off with the pointer: the Record button
  becomes Cancel while it listens, where before it stood there disabled and the
  way out was the keyboard alone.
- The Parameters an Action collects have steps of their own to press, so a
  translation asking for its language can be walked through and backed out of
  without the keyboard. Escape now steps back one Parameter at a time instead of
  abandoning all of them at once.
- The list of Conversations closes the way a menu closes: a click elsewhere or a
  Tab away dismisses it, the click still lands where it was aimed, and its items
  are menu items, so the keyboard reaches them at all.

## 0.1.1 — 2026-09-03

A release about the seconds before an answer, and about a list on Windows that
could not be read.

### The wait

- Where a service takes the instruction, Demysto asks for an answer with no
  chain of thought before it. DeepSeek takes it, and a translation that used to
  spend seconds reasoning now starts straight away.
- Where reasoning arrives anyway it is still no part of the answer, and the
  window says the Model is reasoning instead of standing on "Asking the Model…"
  until the first word.

### Windows

- The open list of a dropdown was white on white. WebView2 paints it from the
  element's own colours, and the form reset had left them transparent.

## 0.1.0 — 2026-09-02

The first release. Demysto sits in the tray, and a Hotkey over whatever you are
looking at turns it into an answer you can keep asking about.

### The loop

- A global Hotkey captures the Selection from the foreground application and
  opens the Palette over it, listing the Actions that accept it. Whatever the
  Capture disturbed on the clipboard is put back.
- An Action may own a Hotkey of its own and run straight away, skipping the
  Palette.
- The answer streams into the result window as Markdown, code blocks
  highlighted, and the Selection it came from is quoted above it.
- The window is a Conversation: follow-up Turns go on about the same Selection.
  Fifty Conversations are held, and the oldest is forgotten first.

### Actions

- Three built-in Actions — explain, translate and summarize — defined the same
  way a user's own are, and running through the same path.
- Any of them can be given an edited prompt, a Model or a personal Hotkey;
  removing the Override brings the built-in definition back.
- An Action declares Parameters and collects them before it runs, which is how
  a translation asks for its target language.
- What an Action says to a Model stays in English whatever the interface speaks.

### Providers and Models

- Several Providers at once over the OpenAI-compatible protocol, with presets
  for openai, deepseek, openrouter, lmstudio and ollama — the last two keyless,
  being servers on this machine.
- Keys come from the settings file, from the environment, or from the variable
  the service's own documentation names.
- A key is verified with a real request to a Model rather than by its shape.
- Actions that bind no Model of their own resolve to the Default Model.

### The desktop

- macOS, Windows, and Linux on X11 and Wayland. On Wayland the Capture is the
  clipboard, and the interface says so rather than pretending otherwise.
- A first run is met by a flow that configures a Provider and ends by inviting a
  press of the Hotkey.
- Tray, macOS menu bar, and an entry in the login items that can be switched
  off.
- The interface speaks the system's language: English, German, Spanish, French
  and Russian.
- A rotating log, reachable from Settings, recording what happened rather than
  what it was about.
- On macOS the Accessibility permission is asked for at the Capture and its
  refusal reported through the Run, so a Capture that fell back to the clipboard
  says so.

### Installation and updates

- A universal `.dmg`, an `.msi` and an NSIS installer, an `.AppImage` and a
  `.deb`.
- Demysto looks for a newer version at startup and once a day after that, offers
  what it finds, and installs nothing until asked. Every artifact is verified
  against a key of Demysto's own.

### Known limits

- The builds carry no Developer ID and no Windows certificate, so the first
  launch warns: on macOS open the application from its context menu once, on
  Windows dismiss SmartScreen once. Neither warning returns.
- Text only. Images and files come in later releases.
- Conversations last as long as the session; nothing is written to disk and
  nothing can be searched afterwards.

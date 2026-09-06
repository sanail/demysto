# Demysto

Demysto is a resident desktop utility that turns whatever the user is looking at — selected text, an image, a file — into an LLM answer in as few keystrokes as possible, and lets the user keep asking about it.

## Language

### What the user acts on

**Selection**:
The input a Run operates on, captured at invocation time. Its kind is one of text, image, or file, and names what a Model is handed rather than where Demysto found it: a picture read from the clipboard and a picture opened from a path reach a Model the same way, so both are image Selections. Where it came from is the Capture's business, not the kind's.
_Avoid_: input, content, context, payload

**Capture**:
The act of obtaining a Selection from the foreground application or the clipboard.
_Avoid_: grab, read, fetch

### What the user invokes

**Action**:
A named, user-runnable operation defined by a prompt template, the Selection kinds it accepts, its parameters, and an optional Model binding. Built-in Actions (explain, translate, summarize, describe image) have the same shape as user-authored ones and run through the same path — there is no privileged built-in variety.
_Avoid_: function, command, tool, skill, feature

**Parameter**:
A value an Action declares and collects before running, beyond the Selection itself — for example the target language of a translation.
_Avoid_: option, argument, setting

**Override**:
A user's change to a built-in Action — an edited prompt, a bound Model, a personal Hotkey. Removing the Override restores the built-in definition.
_Avoid_: customisation, patch, user config

**Palette**:
The window shown by the global hotkey, listing the Actions that accept the current Selection.
_Avoid_: launcher, menu, popup, command bar

**Hotkey**:
A global key combination. One opens the Palette; an Action may additionally own a Hotkey that runs it directly, skipping the Palette.
_Avoid_: shortcut, keybinding, accelerator

### What keeps Demysto running

**Autostart**:
Whether Demysto is in the operating system's login items, so that a resident tool is already running by the time it is wanted. The answer belongs to the system rather than to Demysto: it is asked for rather than remembered, and it changes whenever somebody edits that list in the system's own settings. Demysto offers the choice once in the first-run flow, and holds it in Settings for anybody who changes their mind.
_Avoid_: login item, startup entry, auto-launch, launch at login

**Settings**:
The window in which a user tells Demysto what it needs to know: the Providers and their Models, which Model an Action falls back to, the Palette's Hotkey, the interface language, whether Demysto starts at login. Not the same thing as the Settings File, and holding more than it: the window edits Actions, which are files of their own, and it asks the system questions — Autostart is one — whose answers nothing of Demysto's holds.
_Avoid_: preferences, options, configuration, config window

**Settings File**:
The one file Demysto writes down what it was told in: the Providers, the Default Model, the Default Vision Model, the Palette's Hotkey, the size at which a Selection is called large, the interface language, and whether the first run has happened. Written as text a person may read and edit, which is why Demysto refuses to write over one it could not parse. Actions are not in it.
_Avoid_: config file, config, preferences file, settings

### What answers

**Provider**:
A configured LLM endpoint: protocol, base URL, and credentials. Several may exist at once.
_Avoid_: backend, service, API, vendor

**Model**:
A specific model offered by a Provider, together with the capabilities Demysto needs to know about — notably whether it accepts images.
_Avoid_: LLM, engine, deployment

**Default Model**:
The Model used by any Action that does not bind one of its own.

**Default Vision Model**:
The Model used by any Action whose Selection is an image and that does not bind one of its own. Separate from the Default Model because the cheap everyday Model usually cannot see.

### What results

**Run**:
One execution of an Action against one Selection. A Run produces a Conversation.
_Avoid_: invocation, execution, call, request

**Conversation**:
One Run of an Action plus the follow-up Turns the user takes on the same Selection. The unit the result window shows and the unit history is counted in.
_Avoid_: chat, thread, session, dialogue

**Turn**:
A single user message and the Model's reply within a Conversation.
_Avoid_: message, exchange, round

**Sealed**:
A Conversation that can be read but not added to, because the Selection it was about has been let go. Only a picture is ever let go — one weighs enough that holding every Conversation's would cost a resident tool more memory than it may take — so a text Conversation is never Sealed.
_Avoid_: closed, archived, expired, stale

**Reasoning**:
The chain of thought a Model may produce before its answer. Demysto asks a Provider that takes the instruction not to reason at all, because none of the Actions gains anything from it and the wait is what the user pays. Where reasoning arrives anyway it is never part of the answer and is not kept: the window says only that the Model is working.
_Avoid_: thinking, deliberation, reflection

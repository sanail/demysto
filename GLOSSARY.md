# Demysto

Demysto is a resident desktop utility that turns whatever the user is looking at — selected text, an image, a file — into an LLM answer in as few keystrokes as possible, and lets the user keep asking about it.

## Language

### What the user acts on

**Selection**:
The input a Run operates on, captured at invocation time. Its kind is text or image, and names what a Model is handed rather than where Demysto found it: a picture read from the clipboard and a picture opened from a path reach a Model the same way, so both are image Selections. Where it came from is the Capture's business, not the kind's.
_Avoid_: input, content, context, payload

**Capture**:
The act of obtaining a Selection from the foreground application, the clipboard or the screen.
_Avoid_: grab, read, fetch

### What the user invokes

**Action**:
A named, user-runnable operation defined by a prompt template, the Selection kinds it accepts, its parameters, and an optional Model binding. Built-in Actions (explain, translate, summarize, describe image, custom) have the same shape as user-authored ones and run through the same path — there is no privileged built-in variety.
_Avoid_: function, command, tool, skill, feature

**Parameter**:
A value an Action declares and collects before running, beyond the Selection itself — for example the target language of a translation.
_Avoid_: option, argument, setting

**Override**:
A user's change to a built-in Action — an edited prompt, a bound Model, a personal Hotkey. Removing the Override restores the built-in definition.
_Avoid_: customisation, patch, user config

**Palette**:
The window that lists the Actions accepting the current Selection, opened by the Palette's Hotkey or from the tray. Not named to the user, who has no word for it: the interface calls it the list of Actions, and says where it opens rather than what it is called.
_Avoid_: launcher, menu, popup, command bar

**Hotkey**:
A global key combination. One opens the Palette, another opens it on an area of the screen, and either may be cleared; an Action may additionally own a Hotkey that runs it directly, skipping the Palette.
_Avoid_: shortcut, keybinding, accelerator

### What keeps Demysto running

**Autostart**:
Whether Demysto is in the operating system's login items, so that a resident tool is already running by the time it is wanted. The answer belongs to the system rather than to Demysto: it is asked for rather than remembered, and it changes whenever somebody edits that list in the system's own settings.
_Avoid_: login item, startup entry, auto-launch, launch at login

**Settings**:
The window in which a user tells Demysto what it needs to know: the Providers and their Models, which Model an Action falls back to, the Palette's Hotkey, the interface language, whether Demysto starts at login. Not the same thing as the Settings File, and holding more than it: the window edits Actions, which are files of their own, and it asks the system questions — Autostart is one — whose answers nothing of Demysto's holds.
_Avoid_: preferences, options, configuration, config window

**Settings File**:
The one file in which Demysto writes down what it was told, as text a person may read and edit. Actions are not in it.
_Avoid_: config file, config, preferences file, settings

### What answers

**Provider**:
A configured LLM endpoint: protocol, base URL, and credentials. Several may exist at once.
_Avoid_: backend, service, API, vendor

**Model**:
A specific model offered by a Provider, together with the capabilities Demysto needs to know about — notably whether it accepts images.
_Avoid_: LLM, engine, deployment

**Default Model**:
The Model that serves a text Selection for any Action that binds no Model of its own.

**Default Vision Model**:
The Model used by any Action whose Selection is an image and that does not bind one of its own. Separate from the Default Model because the cheap everyday Model usually cannot see.

### What results

**Run**:
One execution of an Action against one Selection. A Run produces a Conversation.
_Avoid_: invocation, execution, call, request

**Conversation**:
One Run of an Action plus the follow-up Turns the user takes on the same Selection. The unit the result window shows and the unit history is counted in. Called a chat where the user reads it, which is the word they already have for it; Conversation is what the code, this glossary and the documents say, because it names the Run and its follow-ups together rather than the window they happen in.
_Avoid_: thread, session, dialogue

**Turn**:
A single user message and the Model's reply within a Conversation.
_Avoid_: message, exchange, round

**Sealed**:
A Conversation that can be read but not added to, because the Selection it was about has been let go. Only a picture is ever let go, so a text Conversation is never Sealed. Internal: the window never says "sealed" or "let go of", it says the picture is no longer kept and what to do about it.
_Avoid_: closed, archived, expired, stale

**Reasoning**:
The chain of thought a Model may produce before its answer. Never part of the answer and not kept: the window says only that the Model is working.
_Avoid_: thinking, deliberation, reflection

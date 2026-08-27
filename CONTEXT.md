# Demysto

Demysto is a resident desktop utility that turns whatever the user is looking at — selected text, an image, a file — into an LLM answer in as few keystrokes as possible, and lets the user keep asking about it.

## Language

### What the user acts on

**Selection**:
The input a Run operates on, captured at invocation time. Its kind is one of text, image, or file.
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

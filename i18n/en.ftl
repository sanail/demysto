# Demysto's interface in English.
#
# One catalogue per language, and this one is where a message is written first:
# the Rust layer and the three windows both read these files, so the tray menu
# and a notification are never left in a language the rest of the interface is
# not speaking (ticket 14).
#
# Every message here exists in every other catalogue. Nothing enforces that by
# hand — `i18n::tests` reads these files and fails the build over an identifier
# one catalogue has and another does not, and over one the sources ask for and
# no catalogue holds.

## The application itself

app-name = Demysto
tray-open = Open Demysto
tray-actions = Actions
tray-settings = Settings…
tray-quit = Quit Demysto

# macOS only, and only for the key equivalents: `menu` says why the menu bar
# exists at all and why nothing else is on it.
menu-edit = Edit
menu-quit = Quit Demysto

# The one thing Demysto raises a notification for: a Run started from an
# Action's own Hotkey that failed with no window on screen to say so.
notification-stopped-part-way = Demysto stopped part-way through
notification-could-not-answer = Demysto could not answer

## The Actions Demysto comes with
#
# Their names and the Parameters they collect, which is what the Palette shows.
# Their prompt templates stay in `action`, in English, because they are
# addressed to a Model rather than to a person.

action-explain-name = Explain
action-translate-name = Translate
action-translate-target-label = Into which language?
action-summarize-name = Summarize

## The Palette

palette-reading-selection = Reading what you selected…
palette-reading-clipboard = Reading the clipboard…
palette-origin-selection = Selection
palette-origin-clipboard = From the clipboard
palette-nothing-captured = Nothing is selected and the clipboard is empty. Select some text and press the Hotkey again.
palette-filter = Filter Actions…
palette-no-action-matches = No Action is called that.
palette-open-accessibility = Open Accessibility settings
palette-keys-collecting = Enter to run · Esc to go back
palette-keys-choosing = ↑↓ to choose · Enter to run · Esc to close
palette-keys-closing = Esc to close

## The Conversation window

result-conversations = Conversations
result-conversation-unnamed = Conversation
result-nothing-asked-yet = Nothing asked yet.
result-quotation-label = The text this Conversation is about
result-show-more = Show more
result-show-less = Show less
result-asking = Asking the Model…
result-copy-answer = Copy answer
result-copied = Copied
result-stopped = Stopped
result-continue = Continue
result-try-again = Try again
result-ask-another-model = Ask another Model…
result-open-provider-settings = Open { $provider }'s settings
result-open-accessibility = Open Accessibility settings
result-follow-up = Ask a follow-up…
result-stop = Stop
result-ask = Ask
result-keys = Enter to ask, Shift+Enter for a new line, Esc to close

## A rendered code block, whose copy button is markup rather than a component

code-copy = Copy
code-copied = Copied

## Settings

settings-window-title = Demysto Settings
settings-title = Settings
settings-save = Save
settings-saving = Saving…
settings-saved = Saved.
settings-keys = Esc to close
settings-reading = Reading the settings…
settings-unreadable-file = Settings will not write over a file it cannot read, so nothing here can be edited until that file is repaired. Open it, fix what it says, and reopen this window.

### Providers

settings-providers = Providers
settings-add-provider = Add a Provider
settings-remove-provider = Remove this Provider
settings-no-providers = No Provider is configured yet. Add one to start asking things.
settings-provider-name = Name
settings-provider-name-example = openai
settings-provider-service = Service
settings-provider-no-preset = No preset
settings-provider-preset-keyless = { $preset } (no key)
settings-provider-base-url = Base URL
settings-provider-base-url-from-preset = Base URL — leave empty to use the preset's
settings-provider-base-url-example = https://api.example.com/v1
settings-provider-key = API key
settings-provider-key-variable = Or the environment variable holding it
settings-provider-key-variable-example = MY_API_KEY
settings-key-in-file = Held in the settings file — type to replace it
settings-key-in-environment = Taken from { $variable }
settings-key-not-needed = This service has no keys
settings-key-missing = No key yet
settings-key-going = Will be removed when you save
settings-keep-key = Keep the key in the file
settings-remove-key = Remove the key from the file

### The Models one Provider offers

settings-models = Models
settings-fetch-models = Fetch
settings-verify-key = Verify key
settings-add-model = Add a Model
settings-remove-model = Remove
settings-model-sees-images = Sees images
settings-model-verify-with = Verify with
settings-no-models = No Model yet. Fetch the list, or add one by hand.
settings-asking-provider = Asking the Provider…
settings-provider-offers-nothing = It offers no Models.
settings-provider-answered = { $model } answered.

### Defaults

settings-defaults = Defaults
settings-default-model = Default Model — what an Action with no Model of its own uses
settings-default-vision-model = Default Vision Model — what an image uses instead
settings-model-none = None
settings-model-does-not-see = { $model } (does not see)
settings-large-selection = Warn above — how many characters a Selection may hold before Demysto says so
settings-large-selection-default = { $characters } — what Demysto comes with
settings-large-selection-detail = Nothing is ever cut and nothing is ever refused: the warning is there so that an accidental select-all is not silently paid for. Leave the field empty for Demysto's own figure, or set it to 0 to be told nothing.

### Language

settings-language = Language
settings-language-field = The language Demysto speaks
settings-language-follows-system = Follow the operating system
settings-language-detail = Saved by the Save button below, and spoken as soon as it is — the tray menu and this window both.
settings-language-from-environment = { $variable } is set to { $value }, so that is the language Demysto speaks whatever is chosen here.

### Hotkeys

settings-hotkeys = Hotkeys
settings-palette-hotkey = The Palette — what opens it over whatever you are reading
settings-hotkey-record = Record
settings-hotkey-clear = Clear
settings-hotkey-recording = Press a combination… Esc to stop
settings-hotkey-default = { $hotkey } — what Demysto comes with
settings-hotkey-none = None — this Action is reached through the Palette
settings-hotkey-rule = Hold at least one modifier, or press a key that types nothing on its own — F13 and above are the ones most keyboards can send.
settings-palette-hotkey-detail = Saved by the Save button below, and answered to as soon as it is.
settings-action-hotkey-detail = Parameters are not asked for on this path — each takes what it offers.
settings-wayland-hotkeys = Wayland also lets no application claim a Hotkey for itself. Demysto asks the desktop's GlobalShortcuts portal for the combinations below, and the desktop decides what each one answers to — change them in the desktop's own keyboard shortcut settings, where they are listed under Demysto.

### Logs

settings-logs = Logs
settings-logs-detail = Demysto keeps a local log of what it did — which Action, which Model, what went wrong — and never of what you were looking at or what a Model said. Nothing is sent anywhere. Attach these to a bug report.
settings-open-logs = Open the log folder

### Actions

settings-actions = Actions
settings-write-action = Write an Action
settings-actions-detail = Each Action is a file of its own in <code>actions</code>, so one can be backed up or sent to somebody. Built-in Actions are not written there: changing one keeps only what you changed, and resetting it deletes that. An Action is saved on its own, not by the Save button below.
settings-action-changed = Changed
settings-action-yours = Yours
settings-action-edit = Edit
settings-action-reset = Reset
settings-action-delete = Delete
settings-action-name = Name — what the Palette lists
settings-action-name-example = Rewrite plainly
settings-action-model = Model — leave at the default unless this Action needs its own
settings-action-model-default = Whatever the defaults say
settings-action-hotkey = Hotkey — runs this Action on what you have selected, with no Palette in the way
settings-action-prompt = Prompt
settings-action-prompt-example =
    Explain the text below. The text is in {"{{"}selection_language{"}}"}; answer in {"{{"}ui_language{"}}"}.

    {"{{"}selection{"}}"}
settings-action-prompt-detail = <code>{"{{"}selection{"}}"}</code> is what you selected; <code>{"{{"}ui_language{"}}"}</code> and <code>{"{{"}selection_language{"}}"}</code> are the language you read and the one it turned out to be in. Anything else in double braces is a Parameter, which the Palette asks for before the Run — declare it below.
settings-parameters = Parameters
settings-declare-parameter = Declare a Parameter
settings-remove-parameter = Remove
settings-no-parameters = None. This Action runs the moment it is chosen.
settings-parameter-id-example = target
settings-parameter-label-example = Into which language?
settings-parameter-default-example = What it offers
settings-save-action = Save this Action
settings-cancel = Cancel
settings-reset-by-saving = Saving this with nothing changed puts the built-in back.

## The first run
#
# The flow a fresh installation is met by, in the order the spec fixes: confirm
# the language, configure a Provider and prove its key works, walk to the
# Accessibility permission, answer the login-items question, and finish on the
# Hotkey (ticket 15).

welcome-title = Welcome to Demysto
welcome-step = Step { $at } of { $of }
welcome-back = Back
welcome-continue = Continue
welcome-finish = Start using Demysto
welcome-language-title = Demysto found your language
welcome-language-detail = This is the language your operating system says you read. Change it here if it is not, and change it again in Settings whenever you like.
welcome-provider-title = Where the answers come from
welcome-provider-detail = Demysto asks a Model of your own choosing, over your own account. Pick the service, paste the key it gave you, and ask it which Models it offers.
welcome-provider-model = The Model Demysto asks unless an Action says otherwise
welcome-provider-verify-first = The key is put to the Provider before this step is over, so that a wrong one is found now rather than at your first question.
welcome-accessibility-title = Let Demysto read what you have selected
welcome-accessibility-detail = Demysto reads a Selection by sending the copy keystroke to whatever you are reading, and macOS gates that behind the Accessibility permission. Open Privacy & Security → Accessibility and turn Demysto on.
welcome-open-accessibility = Open Accessibility settings
welcome-accessibility-later = Demysto asks macOS about this at every Run, so granting it afterwards works just as well. It is asked for again after an update, which macOS treats as a different application.
welcome-autostart-title = Start Demysto when you log in
welcome-autostart-detail = Demysto waits in the tray for the Hotkey, so it can only answer while it is running. Nothing is registered unless you ask for it here, and your system's own settings take it out again.
welcome-autostart-choice = Start Demysto at login
autostart-refused = Demysto could not change whether it starts at login: { $detail }
welcome-done-title = That is everything
welcome-done-detail = Select some text anywhere and press { $hotkey }. The Palette opens at your cursor listing what Demysto can do with it, and Enter runs the one that is highlighted.
welcome-done-tray = Demysto waits in the tray from now on, and its menu reaches the Palette, the Actions and Settings — the Hotkey is the quick way, not the only one.

## What a Capture could not do
#
# The Palette and Settings say these; the core reports which one happened and
# leaves the sentence to whoever is on screen.

capture-clipboard-only = This is a Wayland session, and Wayland does not let one application type into another. Demysto cannot read what you have selected: copy it yourself with Ctrl+C first, then press the Hotkey, and Demysto reads the clipboard.
capture-clipboard-unavailable = The clipboard is unavailable: { $detail }
capture-keystroke-refused = The copy keystroke could not be sent: { $detail }
capture-no-accessibility = macOS is not letting Demysto read what you selected: Demysto needs the Accessibility permission. Open Privacy & Security → Accessibility and turn Demysto on. macOS withdraws it whenever the application changes, so this can come back after an update.
accessibility-pane-unreachable = Demysto could not open System Settings: { $detail }. The permission is in Privacy & Security → Accessibility.
accessibility-only-macos = Only macOS asks for a permission before Demysto can read what you selected.

## What stopped a Run

run-nothing-to-run = There is nothing to run an Action on: select some text, or copy it, and press the Hotkey again.
run-no-conversation = There is no Conversation to ask this in. Press the Hotkey to start one.
run-no-such-action = There is no Action called "{ $action }". It may have been removed since the Palette opened; press the Hotkey again.
run-nothing-to-retry = There is no Turn to try again. Ask the question again to start a new one.

# The one warning a Conversation carries, said before the Model is asked so that
# it is on screen while the answer is still being paid for.
run-large-selection =
    This Selection is { $shown } { $characters ->
        [one] character
       *[other] characters
    } long, which is over the { $limit } that { $setting } in { $path } is set to. It was sent whole — nothing was cut — so it costs what that costs.

## What the settings file could not be made into

config-unreadable = { $path } could not be read: { $detail }
config-unwritable = { $path } could not be written: { $detail }
config-not-toml-at-line = { $path } is not valid TOML at line { $line }: { $detail }
config-not-toml = { $path } is not valid TOML: { $detail }
config-newer-version = { $path } says it is version { $stated }, and this Demysto understands version { $understood }; update Demysto, or point { $variable } at another directory
config-uneditable = { $path } could not be edited without losing what is written in it, so nothing was saved.
config-no-provider = no Provider is configured; open { $path } and fill in the example it holds
config-in-file = { $reason } in { $path }
config-provider-no-name = a Provider is configured with no name
config-provider-name-has-separator = the Provider "{ $provider }" has a "{ $separator }" in its name, which is what separates a Provider from a Model
config-two-providers-named = two Providers are called "{ $provider }", so a Model of either cannot be named
config-provider-model-no-name = the Provider "{ $provider }" lists a Model with no name
config-provider-model-twice = the Provider "{ $provider }" lists the Model "{ $model }" twice
config-provider-no-base-url = the Provider "{ $provider }" in { $path } states no base_url and no preset to take one from
config-no-key-anywhere = The Provider "{ $provider }" has no API key: set api_key for it in { $path }, or name an environment variable in api_key_env.
config-no-key-export = The Provider "{ $provider }" has no API key: export { $variables }, or set api_key for it in { $path }.
config-no-such-preset = There is no preset called "{ $preset }".

## Which Model a Run resolves to, when it resolves to none

model-none-configured = No Model is configured at all; add one to a Provider there.
model-configured-are = The Models configured there are: { $models }.
model-action-binds-nothing = This Action is bound to the Model "{ $model }", and no Provider in { $path } offers one by that name. { $offered }
model-setting-names-nothing = { $setting } in { $path } names the Model "{ $model }", and no Provider there offers one by that name. { $offered }
model-nothing-nominated = No { $setting } is nominated in { $path }. { $offered }
model-nomination-none-configured = { $setting } names the Model "{ $model }", and no Model is configured at all.
model-nomination-unknown = { $setting } names the Model "{ $model }", and no Provider offers one by that name. The Models configured are: { $models }.

## What a Provider said, or did not

provider-no-connection = Demysto could not open a connection: { $detail }
provider-timed-out = { $provider } did not answer within { $seconds ->
        [one] one second
       *[other] { $seconds } seconds
    }, so Demysto stopped waiting.
provider-unreachable = { $provider } could not be reached: { $detail }
provider-went-quiet = { $provider } went quiet part-way through the answer, so Demysto stopped waiting.
provider-stopped-answering = { $provider } stopped answering part-way through: { $detail }
provider-closed-early = { $provider } closed the connection before the answer was finished.
provider-refused = The Provider refused the request (HTTP { $status }).
provider-refused-saying = The Provider refused the request (HTTP { $status }): { $detail }
provider-malformed = The Provider's answer was not one Demysto could read ({ $reason }): { $body }
provider-no-answer-in-it = it holds no answer

## What an Action could not be made into

action-file-preamble = # An Action Demysto runs. Edit it here, or in Demysto's Settings.
action-needs-name = An Action needs a name to be listed under.
action-needs-prompt = An Action needs a prompt: what it says to the Model, with {"{{"}selection{"}}"} where the Selection goes.
action-accepts-nothing = An Action that accepts no kind of Selection could never appear in the Palette.
action-parameter-needs-name = A Parameter needs a name to be written as {"{{"}like_this{"}}"} in the prompt.
action-parameter-reserved = A Parameter cannot be called "{ $parameter }": that is what a prompt writes to reach something Demysto fills in, so nothing would ever collect it.
action-parameter-needs-label = The Parameter "{ $parameter }" needs a label, which is what the Palette asks for it.
action-parameter-twice = Two Parameters are called "{ $parameter }", so {"{{"}{ $parameter }{"}}"} in the prompt could mean either.
action-binds-nothing-configured = This Action binds the Model "{ $model }", and no Model is configured at all.
action-binds-unknown-model = This Action binds the Model "{ $model }", and no Provider offers one by that name. The Models configured are: { $models }.
action-id-not-a-file-name = "{ $action }" cannot be the name of a file, so no Action can be kept under it.
action-none-to-remove = There is no Action called "{ $action }" to remove. It may already have been deleted; reopen this window.
action-file-newer-version = { $path } says it is version { $stated }, and this Demysto understands version { $understood }. Update Demysto, or take the file out of that directory.
action-file-states-no-field = { $path } states no { $field }. An Action Demysto does not already have must state its name and its template.
action-file-unreadable = { $path } could not be read: { $detail }
action-dir-unreadable = { $path } could not be read, so the Actions in it are not listed: { $detail }
action-file-unwritable = { $path } could not be written: { $detail }
action-file-unwritable-shape = { $path } could not be written as TOML: { $detail }
action-file-invalid-at-line = { $path } is not a valid Action at line { $line }: { $detail }
action-file-invalid = { $path } is not a valid Action: { $detail }

## Hotkeys the desktop would not give up

hotkey-palette-fell-back = { $why } Demysto is using { $hotkey } instead.
hotkey-palette-unclaimable = Demysto could not claim { $hotkey }, the Hotkey that opens the Palette: { $detail }. Another application may already have it. The tray menu reaches everything the Hotkey does.
hotkey-palette-not-a-combination = The settings state the Hotkey "{ $hotkey }" for the Palette, which is not a combination Demysto understands.
hotkey-palette-types-something = The settings state the Hotkey "{ $hotkey }" for the Palette, which is one key that types something. A Hotkey is claimed everywhere, so a key on its own has to be one that reaches nothing you were typing into.
hotkey-palette-refused = The settings state the Hotkey "{ $hotkey }" for the Palette, and Demysto could not claim it: { $detail }. Another application may already have it.
hotkey-action-not-a-combination = { $action } states the Hotkey "{ $hotkey }", which is not a combination Demysto understands. A Hotkey is its modifiers and then one key, written like "Ctrl+Shift+E".
hotkey-action-types-something = { $action } states the Hotkey "{ $hotkey }", which is one key that types something. A Hotkey is claimed everywhere, so a key on its own has to be one that reaches nothing you were typing into — Pause, ScrollLock, PrintScreen, F13 and above, or a volume or media key. Anything else needs a modifier.
hotkey-action-already-held = { $action } states the Hotkey "{ $hotkey }", and { $holder } already has it. Only { $holder } answers to it; give { $action } another.
hotkey-action-refused = { $action } states the Hotkey "{ $hotkey }", and Demysto could not claim it: { $detail }. Another application may already have it.
hotkey-palette-holder = the Palette

## What a Wayland desktop made of the Hotkeys it was asked for

portal-palette-description = Demysto — open the Palette
portal-action-description = Demysto — { $action }
portal-not-taken-yet = The desktop has not taken a Hotkey for { $wanted }, so nothing answers to it yet. Demysto is asking again.
portal-not-taken = The desktop did not take a Hotkey for { $wanted }, so nothing answers to it. Its keyboard shortcut settings are where Demysto's Hotkeys are assigned.
portal-held-under-nothing = The desktop is holding a Hotkey for { $wanted } under no combination, so nothing answers to it yet. Give it one in the desktop's own keyboard shortcut settings.
portal-stopped-answering = The desktop's GlobalShortcuts portal stopped answering, so no Hotkey answers either. Restarting Demysto asks for them again; the tray menu reaches everything the Hotkey does in the meantime.
portal-asking-again = That is what a desktop still coming up looks like: it takes the request for a Hotkey and gives it to nothing, or never answers it at all. Demysto keeps asking for a few minutes, and then leaves the desktop alone.
portal-taken-in-the-end = The desktop took the Hotkeys Demysto asked for when it was asked again.
portal-asked-enough =
    Demysto asked the desktop for its Hotkeys { $asked ->
        [one] once
       *[other] { $asked } times
    } over several minutes, and it did not take them all. It is not asking again until Demysto is restarted — the desktop's own keyboard shortcut settings are where Demysto's Hotkeys are assigned, and the tray menu reaches everything the Hotkey does.
portal-refused = The desktop did not give Demysto the Hotkeys it asked for: { $detail }. Nothing answers to one until it does — its keyboard shortcut settings are where they are assigned, and the tray menu reaches everything the Hotkey does.
portal-unreachable = This is a Wayland session, where Demysto has to ask the desktop's GlobalShortcuts portal for a Hotkey — and it could not reach one: { $detail }. No Hotkey answers. The portal arrives with xdg-desktop-portal, on KDE and on GNOME from version 48. The tray menu reaches everything the Hotkey does.

## The log folder

folder-uncreatable = { $path } could not be created: { $detail }
folder-no-file-manager = Demysto could not open a file manager: { $detail }. The folder is { $path }.

## The settings file a fresh installation is met by
#
# Prose the user reads in their own editor rather than in a window, and
# translated for the same reason the windows are: it is the first thing a new
# installation says, and it says it in a file.

settings-file-preamble =
    # Demysto's settings.
    #
    # Read when Demysto starts, and again whenever Settings writes it — so restart
    # Demysto after editing this file by hand.
    #
    # Uncomment the example below and fill it in.
    #
    # `preset` names a service Demysto knows the conventions of: it fills in
    # `base_url`, and it says which environment variable that service's own
    # documentation tells people to export. State `base_url` yourself for a service
    # that has no preset, or to override what a preset fills in — a local server
    # listening on a port of your own, say.
    #
    # The presets are:
    #
    { $presets }
    #
    # A preset marked "no key" is a server running on this machine, which has no
    # keys at all: a Provider using one needs none, and none is sent. Every other
    # preset wants one.
    #
    # The key is looked for in the variable `api_key_env` names, then in the
    # preset's own variable, then in `api_key` here. Leaving `api_key` out and
    # exporting the variable instead keeps the secret out of this file.
    #
    # `models` lists the Models of a Provider you want to use. `vision` says
    # whether one accepts images, and is stated rather than guessed at from the
    # identifier, because a name is not a capability.
    #
    # A Model is named "<provider>/<model>" wherever one is nominated or bound.
    # `default_model` is what an Action binding no Model of its own resolves to, and
    # `default_vision_model` is what one resolves to for an image.
    #
    # `palette_hotkey` is the key combination that opens the Palette. Leave it out
    # for the one Demysto comes with. It is written as its modifiers and then one
    # key — "Ctrl+Alt+Space" — and a key that types nothing, such as F13, may stand
    # on its own. Settings records one for you if you would rather press it than
    # spell it.
    #
    # `language` is the language Demysto speaks: "en", "de", "es", "fr" or "ru".
    # Leave it out and Demysto follows the operating system, falling back to
    # English. { $languageEnv } overrides both.
    #
    # `large_selection` is how many characters a Selection may hold before Demysto
    # says so in the Conversation. Nothing is ever cut and nothing is ever refused:
    # it is there so that an accidental select-all is not silently paid for. Leave
    # it out for { $largeSelection }, or set it to 0 to be told nothing.
    #
    # `welcomed` is Demysto's own note that the first-run flow has been through.
    # Take the line out to be walked through it again on the next start.
settings-file-preset = #   { $preset }
settings-file-preset-keyless = #   { $preset } (no key)

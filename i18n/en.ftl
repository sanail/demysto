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
tray-update = Update to { $version }…
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
action-describe-image-name = Describe image
action-custom-name = Custom…
action-custom-prompt-label = What should be done?

## The Palette

palette-reading-selection = Reading the selection…
palette-reading-clipboard = Reading the clipboard…
palette-origin-selection = Selection
palette-origin-clipboard = From the clipboard
palette-picture = A picture, { $dimensions }
palette-nothing-captured = Nothing is selected and the clipboard is empty. Select some text and press the Hotkey again.
palette-filter = Filter Actions…
palette-no-action-matches = No Action matches.
palette-back = Back
palette-next = Next
palette-run = Run
palette-open-accessibility = Open Accessibility settings
palette-keys-collecting = Enter to run · Esc to go back
palette-keys-choosing = ↑↓ to choose · Enter to run · Esc to close
palette-keys-closing = Esc to close

## The Conversation window

result-conversations = Chats
result-conversation-unnamed = Chat
result-nothing-asked-yet = Nothing asked yet.
result-quotation-label = The text this chat is about
result-picture-label = The picture this chat is about
result-show-more = Show more
result-show-less = Show less
result-ask-at-original = Ask again at full resolution — { $weight }
result-picture-let-go = This picture is no longer kept.
result-sealed = The picture is no longer kept, so this chat cannot go on. Copy it and press the Hotkey to start a new one.
result-asking = Asking the Model…
result-reasoning = The Model is reasoning…
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
settings-unreadable-file = Nothing can be edited until the settings file is repaired. Fix what it says, then reopen this window.

settings-folder = Settings folder

### The tabs the window is divided into

settings-tab-models = Models
settings-tab-actions = Actions
settings-tab-general = General
settings-tab-about = About
settings-tab-unsaved = unsaved changes
settings-tab-update = an update is ready

### Providers

settings-providers = Providers
settings-add-provider = Add a Provider
settings-remove-provider = Remove this Provider
settings-provider-edit = Edit
settings-provider-unnamed = (unnamed)
settings-no-providers = No Provider yet. Add one to start asking.
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
settings-key-in-file = Held in the settings file
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
settings-verify-which-model = On which Model?
settings-add-model = Add a Model
settings-remove-model = Remove
settings-model-sees-images = Sees images
settings-no-models = No Model yet. Fetch the list, or add one by hand.
settings-asking-provider = Asking the Provider…
settings-provider-offers-nothing = It offers no Models.
settings-provider-answered = { $model } answered.

### Defaults

settings-defaults = Defaults
settings-default-model = Default Model — used unless an Action names its own
settings-default-vision-model = Default Model for pictures
settings-model-none = None
settings-model-does-not-see = { $model } (does not see)
settings-large-selection = Warn above this many characters
settings-large-selection-default = { $characters } by default
settings-large-selection-detail = Nothing is ever cut — the warning is only so that an accidental select-all is not paid for in silence. Leave it empty for Demysto's own figure, or set 0 for no warning.

### Language

settings-language = Language
settings-language-field = Interface language
settings-language-follows-system = Follow the system
settings-language-from-environment = { $variable } is set to { $value }, so that is the language Demysto speaks whatever is chosen here.

### Hotkeys

settings-hotkeys = Hotkeys
settings-palette-hotkey = Opens the list of Actions
settings-hotkey-record = Record
settings-hotkey-clear = Clear
settings-hotkey-cancel = Cancel
settings-hotkey-recording = Press the keys… Esc to cancel
settings-hotkey-default = { $hotkey } — default
settings-hotkey-none = None — run this Action from the list
settings-palette-hotkey-rule = Hold Ctrl, Alt or Shift and press a key. A key that types something cannot be claimed on its own: it would stop typing everywhere.
settings-action-hotkey-rule = Hold Ctrl, Alt or Shift and press a key. A key that types something cannot be claimed on its own: it would stop typing everywhere. Parameters are not asked for on this path — each takes its default.
settings-wayland-hotkeys = On Wayland the desktop hands out Hotkeys, not Demysto. Change them in the desktop's own keyboard settings, where they are listed under Demysto.

### Startup

settings-autostart = Startup
settings-autostart-choice = Start Demysto at login
settings-autostart-changed = Done
settings-autostart-detail = The login items are your system's own list, so this takes effect at once and can be changed there too.

### Logs

settings-logs = Logs
settings-logs-detail = The log records what Demysto did — which Action, which Model, what went wrong — never what you were looking at or what a Model said. Attach it to a bug report.
settings-open-logs = Open the log folder

### Updates

settings-updates = Updates
settings-updates-detail = Every update is signed by Demysto and checked before it is put in place, and nothing is installed until you say so.
settings-version = Demysto { $version }
settings-check-for-update = Check for updates
settings-checking = Checking…
settings-up-to-date = This is the newest version.
settings-update-found = Demysto { $version } is ready to install.
settings-install-update = Install and restart
settings-installing = Installing…

### Actions

settings-actions = Actions
settings-write-action = New Action
settings-actions-detail = Each Action is a file of its own in <code>actions</code> — back one up, or send it to somebody.
settings-action-changed = Changed
settings-action-yours = Yours
settings-action-unsaved = not saved
settings-action-edit = Edit
settings-action-reset = Reset
settings-action-delete = Delete
settings-action-name = Name
settings-action-name-example = Rewrite plainly
settings-action-model = Model
settings-action-model-default = Default
settings-action-hotkey = Hotkey
settings-action-accepts = Runs on
settings-action-accepts-text = Text
settings-action-accepts-image = Pictures
settings-action-accepts-detail = Pick at least one. A picture travels beside the prompt rather than in it, so {"{{"}selection{"}}"} is empty for a picture.
settings-action-prompt = Prompt
settings-action-prompt-example =
    Explain the text below. The text is in {"{{"}selection_language{"}}"}; answer in {"{{"}ui_language{"}}"}.

    {"{{"}selection{"}}"}
settings-action-prompt-detail = <code>{"{{"}selection{"}}"}</code> is what you selected. <code>{"{{"}ui_language{"}}"}</code> is the language you read, <code>{"{{"}selection_language{"}}"}</code> the language it is written in. Anything else in double braces is a Parameter.
settings-parameters = Parameters
settings-declare-parameter = Add a Parameter
settings-remove-parameter = Remove
settings-no-parameters = None — this Action runs as soon as it is chosen.
settings-parameter-id = Name
settings-parameter-label = Question
settings-parameter-default = Default
settings-parameter-id-example = target
settings-parameter-label-example = Into which language?
settings-save-action = Save Action
settings-cancel = Cancel
settings-reset-by-saving = Saving with nothing changed puts the built-in back.

## The first run
#
# The flow a fresh installation is met by, in the order the spec fixes: confirm
# the language, configure a Provider and prove its key works, walk to the
# Accessibility permission, answer the login-items question, and finish on the
# Hotkey (ticket 15).

welcome-title = Welcome to Demysto
welcome-step = Step { $at } of { $total }
welcome-back = Back
welcome-continue = Continue
welcome-finish = Start using Demysto
welcome-language-title = Demysto found your language
welcome-language-detail = Your system says you read this. Change it here, or later in Settings.
welcome-provider-title = Where the answers come from
welcome-provider-detail = Demysto asks a Model over your own account. Pick the service, paste its key, and fetch the Models it offers.
welcome-provider-model = Default Model
welcome-provider-verify-first = Verify the key to go on: better to find a wrong one now than at your first question.
welcome-accessibility-title = Let Demysto read what you have selected
welcome-accessibility-detail = macOS keeps reading your selection behind the Accessibility permission. Open Privacy & Security → Accessibility and turn Demysto on.
welcome-open-accessibility = Open Accessibility settings
welcome-accessibility-later = You can grant it later — Demysto asks at every Run. After an update macOS asks again, treating Demysto as a new application.
welcome-autostart-title = Start Demysto when you log in
welcome-autostart-detail = Demysto waits in the tray and answers only while it is running.
welcome-autostart-choice = Start Demysto at login
welcome-done-title = That is everything
welcome-done-detail = Select some text anywhere and press { $hotkey }. A list of Actions opens at your cursor; Enter runs the highlighted one.
welcome-done-clipboard = Copy some text with Ctrl+C and press { $hotkey }. A list of Actions opens at your cursor; Enter runs the highlighted one.
welcome-done-tray = Demysto now waits in the tray, and its menu reaches everything the Hotkey does.

## What an update could not do

update-refused = Demysto could not ask whether there is a new version: { $detail }
update-install-refused = The update could not be installed: { $detail }
update-nothing-found = Nothing to install: check for updates first.

## What the login items would not do

autostart-refused = Demysto could not change whether it starts at login: { $detail }

## What a Capture could not do
#
# The Palette and Settings say these; the core reports which one happened and
# leaves the sentence to whoever is on screen.

capture-clipboard-only = Wayland does not let one application type into another, so Demysto cannot read your selection. Copy it with Ctrl+C, then press the Hotkey.
capture-clipboard-unavailable = The clipboard is unavailable: { $detail }
capture-keystroke-refused = The copy keystroke could not be sent: { $detail }
capture-no-accessibility = macOS will not let Demysto read your selection without the Accessibility permission. Open Privacy & Security → Accessibility and turn Demysto on.
capture-picture-unreadable = Demysto could not read the picture in the clipboard. Copy it again, or copy a different one.
accessibility-pane-unreachable = Demysto could not open System Settings: { $detail }. The permission is in Privacy & Security → Accessibility.
accessibility-only-macos = Only macOS asks for a permission before Demysto can read your selection.

## What stopped a Run

run-nothing-to-run = Nothing to run an Action on. Select or copy some text, then press the Hotkey again.
run-no-conversation = There is no chat to ask this in. Press the Hotkey to start one.
run-conversation-sealed = The picture this chat is about is no longer kept. Copy it and press the Hotkey to start a new chat.
run-no-such-action = There is no Action called "{ $action }" — it may have been removed. Press the Hotkey again.
run-nothing-to-retry = There is nothing to try again. Ask the question again.

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
model-no-vision-model = No { $setting } is nominated in { $path }, so there is no Model to show a picture to. { $offered }
model-nomination-none-configured = { $setting } names the Model "{ $model }", and no Model is configured at all.
model-nomination-unknown = { $setting } names the Model "{ $model }", and no Provider offers one by that name. The Models configured are: { $models }.

## What a Provider said, or did not

provider-no-connection = Demysto could not open a connection: { $detail }
provider-timed-out = { $provider } did not answer within { $seconds ->
        [one] one second
       *[other] { $seconds } seconds
    }, so Demysto stopped waiting.
provider-unreachable = { $provider } could not be reached: { $detail }
provider-went-quiet = { $provider } went quiet part-way through the answer.
provider-stopped-answering = { $provider } stopped answering part-way through: { $detail }
provider-closed-early = { $provider } closed the connection before the answer was finished.
provider-refused = The Provider refused the request (HTTP { $status }).
provider-refused-saying = The Provider refused the request (HTTP { $status }): { $detail }
provider-malformed = Demysto could not read the Provider's answer ({ $reason }): { $body }
provider-no-answer-in-it = it holds no answer

## What an Action could not be made into

action-file-preamble = # An Action Demysto runs. Edit it here, or in Demysto's Settings.
action-needs-name = An Action needs a name to be listed under.
action-needs-prompt = An Action needs a prompt: what it says to the Model, with {"{{"}selection{"}}"} where the Selection goes.
action-accepts-nothing = An Action that accepts nothing could never be offered. Pick at least one kind of Selection.
action-parameter-needs-name = A Parameter needs a name to be written as {"{{"}like_this{"}}"} in the prompt.
action-parameter-reserved = A Parameter cannot be called "{ $parameter }": Demysto fills that one in itself.
action-parameter-needs-label = The Parameter "{ $parameter }" needs a question to ask for it.
action-parameter-twice = Two Parameters are called "{ $parameter }", so {"{{"}{ $parameter }{"}}"} in the prompt could mean either.
action-binds-nothing-configured = This Action binds the Model "{ $model }", and no Model is configured at all.
action-binds-unknown-model = This Action binds the Model "{ $model }", and no Provider offers one by that name. The Models configured are: { $models }.
action-id-not-a-file-name = "{ $action }" cannot be the name of a file, so no Action can be kept under it.
action-none-to-remove = There is no Action called "{ $action }" to remove — it may already be gone. Reopen this window.
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
hotkey-palette-unclaimable = Demysto could not claim { $hotkey }, which opens the list of Actions: { $detail }. Another application may already have it. The tray menu reaches everything the Hotkey does.
hotkey-palette-not-a-combination = The settings state the Hotkey "{ $hotkey }", which Demysto does not understand. A Hotkey is its modifiers and then one key, written like "Ctrl+Shift+E".
hotkey-palette-types-something = The settings state the Hotkey "{ $hotkey }", which is a single key that types. Add Ctrl, Alt or Shift: on its own it would stop typing everywhere.
hotkey-palette-refused = Demysto could not claim the Hotkey "{ $hotkey }" the settings state: { $detail }. Another application may already have it.
hotkey-action-not-a-combination = { $action } states the Hotkey "{ $hotkey }", which Demysto does not understand. A Hotkey is its modifiers and then one key, written like "Ctrl+Shift+E".
hotkey-action-types-something = { $action } states the Hotkey "{ $hotkey }", which is a single key that types. Add Ctrl, Alt or Shift: on its own it would stop typing everywhere.
hotkey-action-already-held = { $action } states the Hotkey "{ $hotkey }", and { $holder } already has it. Only { $holder } answers to it; give { $action } another.
hotkey-action-refused = { $action } states the Hotkey "{ $hotkey }", and Demysto could not claim it: { $detail }. Another application may already have it.
hotkey-palette-holder = the list of Actions

## What a Wayland desktop made of the Hotkeys it was asked for

portal-palette-description = Demysto — open the list of Actions
portal-action-description = Demysto — { $action }
portal-not-taken-yet = The desktop has not taken a Hotkey for { $wanted }, so nothing answers to it yet. Demysto is asking again.
portal-not-taken = The desktop did not take a Hotkey for { $wanted }, so nothing answers to it. Its keyboard shortcut settings are where Demysto's Hotkeys are assigned.
portal-held-under-nothing = The desktop is holding a Hotkey for { $wanted } under no combination, so nothing answers to it yet. Give it one in the desktop's own keyboard shortcut settings.
portal-stopped-answering = The desktop's GlobalShortcuts portal stopped answering, so no Hotkey answers either. Restarting Demysto asks for them again; the tray menu reaches everything the Hotkey does in the meantime.
portal-asking-again = The desktop is probably still starting up: it takes the request for a Hotkey and gives it to nothing. Demysto keeps asking for a few minutes.
portal-taken-in-the-end = The desktop took the Hotkeys when it was asked again.
portal-asked-enough =
    Demysto asked the desktop for its Hotkeys { $asked ->
        [one] once
       *[other] { $asked } times
    } over several minutes, and it did not take them all. It is not asking again until Demysto is restarted — the desktop's own keyboard shortcut settings are where Demysto's Hotkeys are assigned, and the tray menu reaches everything the Hotkey does.
portal-refused = The desktop did not give Demysto the Hotkeys it asked for: { $detail }. Nothing answers to one until it does — its keyboard shortcut settings are where they are assigned, and the tray menu reaches everything the Hotkey does.
portal-unreachable = Demysto could not reach the desktop's GlobalShortcuts portal, so no Hotkey answers: { $detail }. It arrives with xdg-desktop-portal, on KDE and on GNOME from version 48. The tray menu reaches everything the Hotkey does.

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
    # `palette_hotkey` is the key combination that opens the list of Actions. Leave
    # it out for the one Demysto comes with. It is written as its modifiers and
    # then one key — "Ctrl+Alt+Space". A key that types nothing — a volume or media
    # key, or F13 and above — may stand on its own. Settings records one for you if
    # you would rather press it than spell it.
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

# Demysto v1 — text Actions, Palette, and Conversation

Status: ready for implementation. Vocabulary follows `CONTEXT.md`; decisions
recorded in `docs/adr/0001`–`0005` are binding here and are not re-argued.

## Problem Statement

You are reading something and hit a wall: a term you don't know, a paragraph
that assumes context you lack, a page in a language you don't read, a document
too long to justify reading. Getting an answer today means leaving what you are
doing — switch to a browser or a chat app, open a new conversation, paste,
compose a prompt around the paste, wait, read, switch back. Six or seven
deliberate steps for one small question.

The friction is not the model's latency; it is everything around it. It is high
enough that most of the time you don't ask at all, and simply move on with a
worse understanding than you could have had. And when you do ask, the answer
lands in a chat app that knows nothing about what you were looking at, so a
follow-up question means pasting the context again.

## Solution

Demysto sits resident in the tray. You select text anywhere, press a Hotkey, and
a Palette appears at the cursor listing the Actions that fit what you selected.
One keypress runs one; the answer streams into a Conversation window where the
Selection is already the context, so follow-up Turns cost nothing but typing.

Actions you use constantly get their own Hotkey and skip the Palette entirely —
select, press, read. Actions are not a fixed menu: explain, translate and
summarize are ordinary Actions, and anything you write yourself has exactly the
same shape and the same standing.

## User Stories

### Invoking

1. As a reader, I want to press one Hotkey over selected text and see what I can
   do with it, so that asking a question costs one keystroke instead of an
   application switch.
2. As a reader, I want the Palette to appear at my cursor, so that my eyes do not
   have to travel to find it.
3. As a reader, I want the Palette to list only the Actions that accept what I
   selected, so that I am not choosing from options that cannot run.
4. As a reader, I want to run the highlighted Action with Enter and dismiss the
   Palette with Escape, so that I never need the mouse.
5. As a reader, I want to filter the Palette by typing, so that I can reach an
   Action by name once I have more than a screenful.
6. As a frequent user of one Action, I want to bind a personal Hotkey to it, so
   that the Palette disappears from my most common path entirely.
7. As a reader working in a full-screen application, I want the Palette to appear
   over it, so that the tool works where I actually read.
8. As a reader, I want pressing the Hotkey with nothing selected to fall back to
   the clipboard, so that the tool still works when I copied something a moment
   ago.
9. As a reader, I want pressing the Hotkey with nothing selected and an empty
   clipboard to open the Palette in a state that offers a blank Conversation, so
   that the Hotkey never appears to do nothing.
10. As a user, I want a second press of the Hotkey to close the Palette, so that
    the same key gets me out of what it got me into.
11. As a user, I want the Palette to close when it loses focus, so that it never
    becomes a window I have to manage.

### Getting an answer

12. As a reader, I want the first words of the answer within a moment rather than
    the whole answer after several seconds, so that the tool feels instant even
    when the model is not.
13. As a reader, I want the answer rendered as formatted text — headings, lists,
    code blocks with syntax highlighting — so that a structured explanation reads
    as one.
14. As a reader, I want code blocks to stop flickering while the answer streams,
    so that a partially received code block does not redraw itself repeatedly.
15. As a reader, I want to select any part of the answer with the mouse and copy
    it, so that I can take a sentence or a snippet with me.
16. As a reader, I want a copy button on each code block and on the answer as a
    whole, so that the common case is one click.
17. As a reader, I want to keep asking about the same Selection in the same
    window, so that a follow-up does not require re-establishing context.
18. As a reader, I want to stop a Run in progress, so that an obviously wrong
    answer does not have to finish.
19. As a reader, I want to see which Model answered, so that a disappointing
    answer tells me whether to change the Model or the prompt.
20. As a reader, I want to re-run the same Action against the same Selection, so
    that I can retry after switching Model.

### Actions

21. As a user, I want explain, translate and summarize available out of the box,
    so that the tool is useful before I configure anything beyond a key.
22. As a user, I want to write my own Action from a prompt template, so that the
    tool covers the questions I actually repeat.
23. As an Action author, I want to reference the Selection, the interface
    language and the detected language of the Selection in my template, so that
    one Action behaves correctly across the languages I work in.
24. As an Action author, I want to declare which kinds of Selection my Action
    accepts, so that it does not appear in the Palette when it cannot run.
25. As an Action author, I want to declare Parameters my Action collects before
    running, so that a translation can ask for a target language.
26. As a user, I want to edit a built-in Action's prompt, so that I can adjust the
    wording without recreating the Action from scratch.
27. As a user, I want to reset an edited built-in Action to its original, so that
    experimenting with the prompt is not a one-way door.
28. As a user, I want my Actions to survive an application update, and new
    built-in Actions from that update to appear, so that upgrading is never a
    choice between my work and theirs.
29. As a user, I want each of my Actions stored as its own file, so that I can
    back one up or send it to a colleague.
30. As a user, I want the explanation to arrive in my interface language even
    when the Selection is in another, so that reading foreign material does not
    mean reading a foreign explanation.

### Providers and Models

31. As a user, I want to configure any OpenAI-compatible endpoint, so that I am
    not limited to a vendor list somebody else curated.
32. As a user, I want ready-made presets for the common services, so that setup
    is picking a name rather than looking up a base URL.
33. As a user, I want several Providers configured at once, so that I can keep a
    cheap everyday endpoint and an expensive capable one side by side.
34. As a user, I want to fetch the Model list from a Provider, so that I do not
    have to type Model identifiers from memory.
35. As a user, I want to mark a Model as vision-capable, so that the application
    knows what it can be asked to do without guessing from its name.
36. As a user, I want to nominate a Default Model, so that most Actions need no
    Model configuration at all.
37. As a user, I want to nominate a Default Vision Model separately, so that my
    cheap everyday Model is not asked to look at pictures.
38. As a user, I want to bind a specific Model to a specific Action, so that one
    expensive Action does not force every Action onto an expensive Model.
39. As a user, I want to keep my key in an environment variable instead of the
    settings file, so that my configuration can be committed or shared without
    the secret in it.
40. As a user, I want an environment variable to win over the settings file, so
    that I can override a key for one launch without editing anything.
41. As a user, I want the settings file created with owner-only permissions, so
    that the trade being made on my behalf is at least made carefully.
42. As a user, I want my key tested against the Provider during setup, so that I
    learn it is wrong immediately rather than at the first Run.

### When things go wrong

43. As a user, I want an API error shown inside the Conversation with the
    Provider's own message, so that I can act on what actually happened.
44. As a user, I want a retry button on a failed Run, so that a transient failure
    costs one click.
45. As a user, I want an authentication failure to offer me the relevant
    Provider's settings directly, so that the fix is where the problem is
    reported.
46. As a user, I want a stream that breaks mid-answer to keep the text it already
    delivered and offer to continue, so that a network hiccup does not discard a
    partial answer.
47. As a user, I want a failure of an Action launched by its own Hotkey to reach
    me as a system notification, so that a failure with no window on screen is
    not silent.
48. As a user, I want to be told when a Selection is unusually large before I
    spend tokens on it, so that an accidental select-all is not an expensive
    mistake.

### Living in the system

49. As a user, I want the application to live in the tray without cluttering my
    dock or taskbar while idle, so that a resident tool stays out of the way.
50. As a user, I want a dock or taskbar entry while a Conversation, Settings or
    the first-run window is open, so that I can switch back to one with the same
    keys I use for every other window.
51. As a user, I want to reach Settings and run Actions from the tray menu, so
    that the tool is usable when I don't remember the Hotkey.
52. As a user, I want to be asked once whether to start Demysto at login, so that
    it neither installs itself silently nor stops working after a reboot.
53. As a user, I want launching a second copy to raise the Palette instead of
    starting a second instance, so that the tool cannot end up fighting itself.
54. As a macOS user, I want to be walked to the Accessibility permission during
    setup, so that the central feature is not silently broken from the first run.
55. As a macOS user, I want a clear message when that permission is missing at
    the moment I run an Action, so that a revoked permission looks like a
    permission problem rather than a broken application.
56. As a Wayland user, I want to be told plainly that capturing the Selection is
    unavailable and that copying manually still works, so that I understand the
    limitation instead of concluding the tool is broken.
57. As a new user, I want a first-run flow that ends with a working Action, so
    that the first thing I experience is the tool doing its job.

### Language, privacy, upkeep

58. As a user, I want the interface in my operating system's language when it is
    supported, and English when it is not, so that setup requires no decision.
59. As a user, I want to change the interface language in Settings, so that the
    operating system's choice is not final.
60. As a Russian speaker, I want counts and quantities to read grammatically, so
    that the interface does not read as machine-translated.
61. As a privacy-conscious user, I want nothing sent anywhere except my chosen
    Provider, so that a tool holding my key and my screen contents earns the
    access it asks for.
62. As a user, I want history kept only until I quit, so that what I looked at
    today is not sitting on disk next month.
63. As a user, I want to reach the log files from Settings, so that I can attach
    them to a bug report.
64. As a user, I want the application to update itself, so that staying current
    is not a chore I have to remember.
65. As a Linux and Windows user, I want a build for my platform from the same
    release, so that the tool is not quietly macOS-first.

## Implementation Decisions

### Shape

The entire product logic lives in a Rust **core** that references no Tauri types.
The Tauri command layer is a set of thin adapters over the core's public API, and
the frontend holds no logic beyond rendering — with one exception, the first-run
flow, which owns the order of its own steps and what each of them saves. What it
saves still goes through the core's façade like every other save. This is what
makes a single test seam possible; see *Testing Decisions*.

Streaming reaches the frontend over a Tauri channel. Per ADR-0001 the frontend is
Svelte 5 with Vite and Tailwind v4, and Markdown is rendered by `markdown-it`
with raw HTML disabled — model output is untrusted content — with syntax
highlighting by `highlight.js` over a reduced language set.

### Core modules and their responsibilities

**Config.** Reads and writes a single TOML file in the platform config directory,
created mode `0600` on Unix, carrying a `version` field for future migrations.
Owns key resolution in the order fixed by ADR-0002: a Provider's `api_key_env`
field, then the preset's conventional variable, then the file's `api_key`. Keys
are read once at startup; the environment is snapshotted here and nowhere else,
and everything that has a say in it — the language override included — reads
that snapshot. The one exception is the session type, which `desktop` asks for
because it decides which Capture exists at all.

**Action catalogue.** Built-in Actions are compiled into the binary; disk holds
only user-authored Actions, one file each, plus Overrides of built-in ones
(ADR-0005). The catalogue's job is to produce the merged, effective set of
Actions: built-ins with their Overrides applied, plus user Actions. Deleting an
Override restores the built-in.

**Model resolution.** Given an Action and a Selection, resolve to one Model:
the Action's explicit binding, else the Default Vision Model when the Selection is
an image, else the Default Model. An unresolvable binding is a first-class error
that names what is missing and which setting fixes it — not a generic failure.

**Prompt assembly.** Renders an Action's template against `{{selection}}`,
`{{ui_language}}`, `{{selection_language}}` and the Action's declared Parameters.
Built-in explain and summarize are seeded to answer in `{{ui_language}}`; a
translation's target language is a Parameter, not a variable.

**Provider adapter.** One implementation of the OpenAI Chat Completions contract,
parameterised by base URL and credentials — every supported service is that
contract with a different address. Streaming via SSE. Owns timeouts and
cancellation. This is the only module that performs network I/O.

**Conversation store.** In-memory only for v1, capped at 50 Conversations with
oldest-first eviction, lost on quit without a confirmation prompt. Closing a
Conversation window hides it; it stays in the list.

**Stream assembly.** The core accumulates streamed text, closes unterminated code
fences, throttles at roughly 50 ms, and emits a render-ready string. This is
deliberately in Rust rather than the frontend: it moves a presentation concern
across a layer in exchange for collapsing the test surface to one language. Full
accumulated text crosses the channel each tick rather than a delta; at realistic
answer sizes the bandwidth is immaterial.

**Capture.** A trait with per-platform implementations, so the core is testable
against a fake. The real implementation synthesises a copy keystroke, reads the
clipboard, and restores the previous clipboard contents; it falls back to
whatever the clipboard already held when the contents do not change. On Wayland
it degrades to clipboard-only per ADR-0003, detected via `XDG_SESSION_TYPE`.

**Errors.** Surfaced as entries within a Conversation, never as modal dialogs,
carrying the Provider's own message plus a retry affordance and a Model switch.
Authentication failures link to the offending Provider's settings. A failed Run
launched from an Action's own Hotkey, with no window on screen, raises a system
notification instead.

### Shell and platform

The Palette is an `NSPanel` on macOS via `tauri-nspanel` — an ordinary window
will not float over full-screen applications, which is where the tool is most
often used — and an always-on-top, taskbar-skipping window elsewhere. The
Conversation and Settings windows are ordinary windows.

macOS activation policy is dynamic: accessory while only the Palette is on
screen, regular while a Conversation, Settings or the first-run window is open,
back to accessory when the last one closes. The tray icon is always present and
its menu reaches the Palette, the Actions, and Settings, so the mouse-only path
exists independently of the dock.

Single instance is enforced; a second launch raises the Palette. Autostart is
offered once during first-run setup, never enabled silently.

First-run order, in a window of its own: confirm the detected language,
configure a Provider with a key and a Model, verify it with a live request,
request the macOS Accessibility permission with a button that opens the correct
settings pane, offer autostart, and finish on an invitation to try the Hotkey.
The flow is over when that window closes, whichever way it closes, and the
settings file records that so it does not come back — ADR-0013. Accessibility is
re-checked at every Run, because macOS revokes it whenever the binary's
signature changes.

### Interface language

One Fluent catalogue per language in `i18n/` is the single source of truth,
consumed by both the frontend and the Rust layer — the tray menu and
notifications are native, so Rust needs the same strings. Vite imports the files
as text and `demysto-core` compiles them in with `include_str!`; neither derives
its strings from the other. Fluent rather than flat JSON because Russian plural
forms are exactly what it exists for.

English, German, Spanish, French and Russian ship. Another is a catalogue and a
variant of `Interface`, which the compiler then walks through everything that
matches on one. Nothing else is left to remember: the suite fails the build over
a message the new catalogue does not hold, over one that names a variable
English does not, and over the two lists the frontend writes by hand — its map
of catalogues and the one every window draws its language field from —
disagreeing with `Interface::ALL`.

Language follows the operating system, falls back to English, and is overridable
in Settings and by `DEMYSTO_LANGUAGE`, in that order of precedence: the variable,
then the settings file, then the system. A source naming a language Demysto does
not speak is passed over rather than treated as English, so a typo in the
variable still leaves the desktop deciding. Changing it in Settings takes effect
without a restart, in every window and in the tray menu.

What the built-in Actions say to a Model stays in English whatever the interface
speaks — ADR-0012.

### Distribution

Per ADR-0004: the Tauri bundler, a universal macOS artifact, `.dmg` / `.msi` and
NSIS / `.AppImage` and `.deb` from a GitHub Actions matrix, on an Ubuntu runner
old enough that its glibc does not become the floor for supported distributions.
In-app updates through `tauri-plugin-updater` against a manifest in GitHub
Releases, with its keypair generated before the first release. macOS builds are
signed with a self-signed certificate rather than a Developer ID, which is what
keeps an update from revoking the Accessibility permission (ADR-0015).

No telemetry, no crash reporting, no analytics of any kind. Local logs with
rotation, reachable from Settings.

## Testing Decisions

**One seam: the core's public API.** Tests call the same façade the Tauri
commands call, substitute the outside world at its edges, and assert on what
comes back. Nothing asserts on internal structure, module boundaries, or call
sequences — a test that would fail on a refactor that changed no behaviour is a
test written at the wrong level.

The two substitutions at that seam are a mock HTTP server standing in for the
Provider, and a temporary directory standing in for the config location. Capture
is substituted through its trait.

What is tested there:

- Key resolution across all three sources and their precedence, including the
  case where the file has a key and the environment overrides it.
- The effective Action set: a built-in alone, a built-in with an Override, an
  Override removed, a user Action, and a user Action colliding with a built-in
  name.
- Model resolution down the full chain, including the unresolvable case and the
  quality of the error it produces.
- Prompt assembly for every variable and for declared Parameters.
- Request construction against the recorded mock request, so that what is sent
  to a Provider is asserted rather than assumed.
- SSE parsing, including chunk boundaries falling mid-event.
- Stream assembly: unterminated code fences closed at every intermediate state,
  throttling, and the final state matching the unthrottled concatenation.
- Cancellation of a Run in progress.
- Error paths: a 401, a 500, a malformed body, a connection dropped mid-stream,
  and a timeout — each asserted to produce the specific error the interface
  needs, not a generic one.
- Conversation accumulation across Turns and eviction at the cap.

**No prior art.** This is the first code in the repository, so these tests set
the pattern rather than follow one. That is a reason to be deliberate about the
first module written, not a reason to defer the decision.

**Not in the suite.** Capture's real implementations, global Hotkeys, the tray,
`NSPanel` behaviour, activation policy, the login items, and the Accessibility
permission flow are interactive operating-system machinery, checked on a live
desktop per platform rather than in the suite. So is the first-run flow's own
order of steps, which is the one piece of sequencing that lives in a window
rather than in the core; what it writes goes through the same façade every other
save does, and that is where it is tested. No WebDriver end-to-end suite:
`tauri-driver` has no macOS support, and building one for two platforms out of
three does not pay for itself on an application with two screens.

## Out of Scope

Images and vision (v1.1) and files (v1.2), along with everything they imply:
chunking, map-reduce summarisation, and file type detection. Large Selections are
warned about and sent as-is; nothing is truncated or split.

History on disk, and with it search across past Conversations. Keychain storage
(ADR-0002). Notarization, a Windows signature, the Homebrew cask and a `winget`
manifest — one later milestone gated on certificates nobody has bought, not four
(ADR-0004).

The RemoteDesktop portal on Wayland (ADR-0003). Multiple simultaneous
Conversation windows. Importing and sharing Action presets as a feature, though
the on-disk layout is chosen so as not to preclude it. Telemetry in any form.
Mobile, web, and the App Store.

## Further Notes

Two commitments are bound to the calendar rather than to readiness. The updater
keypair and the macOS signing certificate must both exist before the first
release: changing either afterwards costs every installed copy something it
cannot get back on its own — a path forward in the first case, the Accessibility
permission in the second (ADR-0015). And Homebrew disables casks failing
Gatekeeper in its main repository from 2026-09-01 and has withdrawn the
`--no-quarantine` escape, so the Developer ID must precede any Homebrew
distribution rather than follow it.

The one decision in this spec that is a deliberate deviation from clean layering
is stream assembly living in Rust. It is recorded here rather than as an ADR
because it is cheap to reverse: moving fence-closing back to the frontend is a
change to one module and one test file. If it is ever reversed, the frontend
acquires a second test seam, and that is the cost being tracked.

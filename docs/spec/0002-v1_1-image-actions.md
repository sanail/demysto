# Demysto v1.1 — image Selections

Status: ready for implementation. Vocabulary follows `CONTEXT.md`; the decisions
recorded in `docs/adr/0001`–`0017` are binding here and are not re-argued. This
milestone builds on `docs/spec/0001`, which stands unchanged except where this
document says otherwise. User stories continue that spec's numbering.

## Problem Statement

Demysto answers questions about what you are looking at, as long as what you are
looking at is text. Half of what anyone reads is not: a diagram in a paper, a
screenshot a colleague pasted into a chat, an error dialog, a chart with no
caption, a page of a scanned document. For all of it the tool is not merely
worse than a chat app — it is absent, because there is nothing to select.

The workaround is the one Demysto exists to abolish: save or copy the picture,
switch to a browser, open a conversation, attach, compose a prompt around the
attachment, wait, read, switch back. Longer than the text path it replaced,
because an attachment is more work than a paste.

## Solution

Copying a picture makes it a Selection like any other. The same Hotkey opens the
same Palette, which now shows a thumbnail of what it caught and lists the
Actions that accept an image — out of the box, one: **describe image**. The
answer streams into the same Conversation window, with the picture above the
first Turn, and follow-up Turns cost nothing but typing, exactly as they do for
text.

The picture is fitted before it is sent, so that a full-screen capture does not
cost what a full-screen capture weighs. Where that loses something the answer
needed, the Conversation offers to ask again at the original resolution, with
the price of doing so written on the button.

## User Stories

### Capturing a picture

68. As a reader, I want the Hotkey I already press over text to work over a
    picture I have copied, so that there is one thing to remember rather than
    two.
69. As a reader, I want the Palette to show me the picture it caught, so that I
    can see it is the right one before I spend anything on it.
70. As a user, I want the picture that was on my clipboard to still be on it
    afterwards, so that a tool that reads my clipboard does not also empty it.
71. As a user copying a spreadsheet cell or a fragment of a page — where the
    clipboard ends up holding both text and a picture — I want the text, so
    that the common case is not silently turned into a photograph of itself.
72. As a Wayland user, I want a copied picture to work the same way a copied
    piece of text does, so that the one platform without a synthetic copy is
    not also the one without pictures.

### Asking about it

73. As a reader, I want a **describe image** Action out of the box, so that the
    feature is useful before I write anything of my own.
74. As a reader, I want the description in my interface language whatever is
    written in the picture, so that reading foreign material does not mean
    reading a foreign explanation.
75. As a reader, I want the picture shown above the first Turn in the
    Conversation, so that a third question is not asked blind.
76. As a reader, I want to keep asking about the same picture in the same
    window, so that "and what does the arrow on the left mean?" is one line of
    typing.
77. As a reader whose answer missed a detail, I want to ask again at the
    original resolution, so that fine print in a screenshot is recoverable.
78. As a reader, I want to see what asking again will weigh before I press it,
    so that the cost is on the action rather than in a warning over it.
79. As a reader who has asked again, I want the rest of that Conversation to
    stay at the original resolution, so that the second question is not answered
    worse than the first.

### Models

80. As a user with no vision-capable Model configured, I want to be told which
    setting fixes that rather than shown a confident answer from a Model that
    cannot see, so that a wrong answer is not indistinguishable from a right
    one.
81. As a user, I want that message reported inside the Conversation with a way
    to the setting, so that the fix is where the problem appears.
82. As a user who nominated a Model I did not tick as vision-capable, I want
    Demysto to send the picture anyway, so that a flag I fill in by hand does
    not become a thing that forbids me.

### History and memory

83. As a reader, I want to come back to an earlier picture Conversation and
    read what I was told, so that closing a window does not throw away the
    answer.
84. As a reader, I want to be told plainly when an earlier picture Conversation
    can no longer be continued, so that an input box that will not work does not
    look broken.
85. As a user, I want a tool that lives in my tray not to grow without bound as
    I ask about pictures, so that leaving it running all week costs nothing.

### Authoring

86. As an Action author, I want to write an Action that accepts pictures, so
    that "describe image" is a starting point rather than the whole of it.
87. As an Action author, I want to write one Action that accepts both text and
    pictures, so that a single prompt can cover both where it makes sense.

## Implementation Decisions

### Selection

`Selection` gains one variant. `Kind::Image` already exists, because Model
resolution was built and tested against it in v1; nothing about that chain
changes here.

The kind names what a Model is handed, not where Demysto found it — recorded in
`CONTEXT.md`. That is what settles, ahead of v1.2, that a picture opened from a
path is an image Selection and not a file one.

An image Selection carries the picture twice: **fitted**, which is what a Run
sends, and **original**, which is what asking again sends. Both are PNG bytes.
Fitting is a ceiling of 1568 px on the longest side, preserving aspect ratio and
never enlarging; PNG rather than JPEG because the case that matters most is text
on a screenshot, and that is the case JPEG's artefacts land on. A picture already
within the ceiling is fitted and original at once, and is stored once.

`Selection::as_text()` returns an empty string for an image. Prompt assembly
renders `{{selection}}` and `{{selection_language}}` empty rather than refusing,
because an Action declaring `accepts = ["text", "image"]` is legitimate and its
template must reference `{{selection}}` for the text half. **The picture is never
substituted into the text of a prompt.** It travels as its own content part, and
nothing in prompt assembly may learn otherwise.

### Capture

One Hotkey, one Capture, unchanged in shape: read what the clipboard holds, send
the synthetic copy, read again, take what changed, put back what was there. What
changes is that "what the clipboard holds" now means text or a picture.

`arboard` is rebuilt with its `image-data` feature, which is what makes
`get_image` and `set_image` exist at all; it pulls `objc2-core-graphics` on
macOS and `windows-sys/Win32_Graphics_Gdi` on Windows. It hands over raw RGBA8
with a width and a height — not encoded bytes — so encoding to PNG happens in
Demysto whether or not anything is fitted.

Change is detected by a signature of width, height and a hash of the buffer, not
by comparing buffers: a Hotkey press must not walk megabytes twice. Restoring
uses `set_image` where a picture was displaced, so the guarantee v1 gave for
text now covers pictures.

**Text wins.** Where the clipboard holds both — a spreadsheet cell, a fragment
of a page, a Word selection — the text is the Selection. The picture is read
only where v1 returned `Nothing`: an empty clipboard, or one holding nothing
meaningful. The reasoning is that the ambiguous sources almost always mean the
text and carry the picture as a secondary flavour, while the unambiguous ones —
a screen capture, "copy image" — put no text on the clipboard at all, so the
picture loses nothing by yielding.

Wayland keeps its clipboard-only Capture (ADR-0003) and needs no other change:
reading a picture from the clipboard is the half that works there.

### What is sent

`Message.content` stops being a string. It becomes either a string or an ordered
list of parts — text, and `image_url` carrying a `data:image/png;base64,` URL —
which is the OpenAI Chat Completions shape every supported service implements.
A text Run keeps sending a bare string, so nothing about v1's recorded requests
changes.

The picture rides in the **first** user message. Because the whole message list
is resent on every Turn, and the contract holds no state, the picture is in
front of the Model for every question in the Conversation. That is the cost of
following up on a picture, and it is deliberate: the alternative is a second
question answered from the Model's memory of its own first answer.

Threading this reaches six places that are `String` or `&str` today —
`Action::prompt`, `Turn.prompt`, `Conversation::said`, `Store::asking`,
`provider::answer` and `provider::asking` — and the change is theirs, not the
Provider adapter's. Nothing about SSE parsing, streaming, timeouts or
cancellation is touched.

### Model resolution

The chain is v1's and stays: an Action's binding, else the Default Vision Model
for an image, else the Default Model. What changes is the last step. An image
with **no** Default Vision Model nominated is now a first-class error naming
`default_vision_model`, rather than falling through to a Model that probably
cannot see; that fall-through was v1's placeholder and is removed.

A Default Vision Model that *is* nominated is used whether or not it is ticked
vision-capable. The tick is stated by a person, not discovered, and a field
somebody fills in by hand is not grounds for refusing them.

The error is reported the way every other Run error is (spec 0001): an entry
inside the Conversation, with the Provider's own message where there is one, a
retry, and a link to the setting. The Action is **not** hidden from the Palette
on account of it — an Action that vanishes reads as "this tool cannot do
pictures", where the true answer is "nominate a Model, here".

### The built-in Action

One new built-in, `describe image`, declaring `accepts = ["image"]` and no
Parameters. It brings the count to the four ADR-0005 named. Its template is
English (ADR-0012) and asks for the answer in `{{ui_language}}`, as explain and
summarize already do.

One and not two: "read the text in this picture" is a follow-up Turn in the
Conversation that is already open, the Palette is worth less the longer it gets,
and a built-in is ten lines of `action.rs` plus a message in five catalogues if
it turns out to be wanted.

### Windows

The **Palette** shows a thumbnail where it shows two clamped lines of text for a
text Selection, in the same box and at the same height, cropped across rather
than resized down — the Palette must not change size according to what was
caught. Its only job is confirming this is the right picture, which a line
reading "Image 1280×720" does not do.

The **Conversation** window shows the picture above the first Turn, and a button
to ask again at the original resolution carrying the weight it will send
("Ask again at original resolution — 7.4 MB"). Pressing it re-runs the Action on
the same Selection, and every Turn after it in that Conversation sends the
original too: a person who asked for resolution asked because of a detail, and
the question after that one is about the same detail.

There is **no setting** governing this, in Settings, in the Palette or in an
Action file — ADR-0017.

The **history list** shows an image Conversation as its dimensions, since
`Conversation::preview` and `Summary::about` are built from `as_text()` and an
image has none.

`tauri.conf.json`'s CSP gains `img-src 'self' data:` and nothing else; without it
neither thumbnail renders, and with anything wider the window could load a
picture from somewhere Demysto did not put there.

### Memory, and sealing a Conversation

A picture is heavy in a way text is not, and the Conversation store holds fifty.
The rule is that a picture lives as long as the window that shows it:

- Closing the result window releases the pictures of every Conversation in it.
  This is a signal the core does not hear today — nothing in `Store` reacts to a
  window closing — and the shell must send it.
- A ceiling of 128 MB over held pictures releases the oldest first, so that a
  window left open all week cannot grow without bound. In any ordinary session it
  never fires.

Releasing a picture does **not** discard the Conversation. The exchange stays in
the list and reads exactly as it did; what is gone is the ability to add to it,
because there is no longer anything to resend. `CONTEXT.md` calls a Conversation
in that state **Sealed**, and the window says so where the input box was, rather
than offering a box that would fail.

Only an image Conversation is ever sealed. A text Conversation keeps its
Selection until the session ends, as in v1.

### There is no size threshold for a picture

v1 warns before a Run on an unusually large text Selection. A picture gets no
equivalent: fitted, it is never large enough to be worth a warning, and the one
path that sends something heavy — asking again at the original resolution — is
an explicit press whose weight is written on the button. A price on the action
beats a warning over it.

### Dependencies

Two, both in the core. `image`, with default features off and `png` enabled: it
resizes and encodes, and it does not need to decode anything, because the
clipboard hands over pixels rather than a file. And a base64 encoder for the
data URL. No dialog, filesystem or clipboard Tauri plugin is added — the
capability set stays `core:default` and `notification:default`.

## Testing Decisions

The seam is v1's: the core's public API, with a mock HTTP server for the
Provider, a temporary directory for the config, and the `Capture` trait
substituted by a fake. The fake's `Desktop` seam is `Option<String>` today and
grows a picture alongside it.

What is tested there:

- Capture with a picture on the clipboard, with text on it, with both — text
  winning — and with neither.
- The clipboard being restored after a Capture displaced a picture, and the
  signature detecting a change without comparing buffers.
- Fitting: a picture over the ceiling, one under it, one exactly on it, and a
  very wide one, each asserted on the resulting dimensions and on aspect ratio.
- Request construction against the recorded mock request: the parts array, the
  data URL's prefix, the picture riding in the first user message, and a text
  Run still sending a bare string.
- The picture present in every Turn's request, and the original taking over from
  the fitted one after an ask-again and staying for the Turn after that.
- Model resolution for an image with no Default Vision Model nominated,
  asserting the error names `default_vision_model`; and with one nominated that
  is not ticked vision-capable, asserting it is used.
- Prompt assembly for an image: `{{selection}}` and `{{selection_language}}`
  empty, and an Action accepting both kinds rendering correctly for each.
- The effective Action set including the new built-in, and a user Action
  declaring `accepts = ["image"]` surviving a save.
- Sealing: a released picture leaving the Conversation readable, a follow-up on
  a sealed Conversation refused with the error the window needs, and the ceiling
  releasing oldest first.

Not in the suite, for v1's reasons: the real clipboard on each platform, the
thumbnails, and the result window's close signal. Those are checked on a live
desktop per platform, on all three, per the standing rule that a
cross-platform change is verified on each rather than inferred from one.

## Out of Scope

**A path to a picture**, though it has been asked for since Demysto was first
described. There is no entry point for a path today — command-line arguments
are discarded, no drag-and-drop is listened for, and no dialog plugin is
present — so the only route would be a copied string that happens to be a path.
That makes one Capture ambiguous: a string that is both a path to a picture and
a piece of text either drops explain, translate and summarize from the Palette,
or forces a Capture to carry two Selections at once, which `Demysto::actions`
and `Demysto::run` are built around not doing. The ambiguity is real and worth
solving once, for every file type, in v1.2 — not twice, the first time for one
extension.

Everything v1.2 already owns: files, chunking, map-reduce summarisation, file
type detection.

Capturing a region of the screen. It is a different capability with different
permissions — Screen Recording on macOS, another pane in the first-run flow —
and it does not fit the definition of Capture, which is about the foreground
application and the clipboard.

Multiple pictures in one Selection. Pictures in a follow-up Turn: the picture of
a Conversation is the one it opened with.

History on disk, and with it pictures surviving a quit. Everything else spec
0001 put out of scope stays there.

## Further Notes

`arboard`'s `image-data` feature is the one change here that reaches all three
platforms' build graphs at once, and it is the one to build on all three before
anything else in this milestone is written. A feature that fails to compile on
Windows is a worse thing to discover after the core is threaded for pictures
than before.

Fitting is a constant. If photographs turn out to matter more than screenshots,
the format is the same one line as the ceiling; that is the reason both are
written down as numbers here rather than argued as principles.

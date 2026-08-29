/**
 * Hotkeys as this window reads and writes them.
 *
 * A Hotkey is stored as its modifiers and then one key, joined by `+`, and one
 * dialect is written throughout: `Ctrl+Alt+Shift+R`, which is the form the
 * backend's own guidance teaches and the form somebody hand-editing an Action
 * file would write. The keyboard reports the key as `KeyR`, and the backend
 * accepts either, so the shortening happens here — at the one point where a
 * Hotkey is written down.
 *
 * Reading a pressed key and writing a Hotkey for the eye are both this window's
 * work: the key arrives as a browser event that exists nowhere else, and asking
 * the backend about every keystroke of a recording would be a round trip per
 * press. What a combination *means* — whether it parses, whether anything else
 * already has it — is never decided here; the backend claims it and says.
 */

/**
 * The codes of the modifier keys themselves.
 *
 * A combination is its modifiers and then one other key, so these are never the
 * key: holding Shift while reaching for the rest of a combination must not be
 * mistaken for finishing it.
 */
const MODIFIER_KEYS = /^(?:Control|Alt|Shift|Meta|OS)(?:Left|Right)?$/;

/**
 * The modifiers a Hotkey is written with, in the order they are written in, and
 * what each is called where the user reads it.
 *
 * Two spellings each, because a file written by hand may use any the backend
 * accepts, and because macOS calls two of these something else. `Ctrl` is
 * `Ctrl` everywhere: it is the one modifier whose name nobody disputes.
 */
const MODIFIERS = [
  { flag: "ctrlKey", written: "Ctrl", mac: "Ctrl" },
  { flag: "altKey", written: "Alt", mac: "Option" },
  { flag: "shiftKey", written: "Shift", mac: "Shift" },
  { flag: "metaKey", written: "Super", mac: "Cmd" },
] as const;

/** Every spelling of a modifier the backend accepts, by what it is read as. */
const SPELLINGS: Record<string, (typeof MODIFIERS)[number]> = {
  control: MODIFIERS[0],
  ctrl: MODIFIERS[0],
  alt: MODIFIERS[1],
  option: MODIFIERS[1],
  shift: MODIFIERS[2],
  super: MODIFIERS[3],
  cmd: MODIFIERS[3],
  command: MODIFIERS[3],
};

/**
 * The combination just pressed, or `null` while it is not one yet.
 *
 * `null` rather than a Hotkey for a key held on its own, unless it is one of
 * `alone`: a Hotkey is claimed everywhere, so binding a bare letter is a way to
 * lose that letter rather than to bind an Action. Which keys escape that is the
 * backend's answer — see its `hotkey` module — and it is passed in rather than
 * held here so that there is no moment where this has been asked and not yet
 * told.
 */
export function combination(
  event: KeyboardEvent,
  alone: ReadonlySet<string>,
): string | null {
  // A press with no code behind it is not a key anything could be claimed on:
  // some input methods send one while they are composing.
  if (!event.code || MODIFIER_KEYS.test(event.code)) return null;

  const held = MODIFIERS.filter((modifier) => event[modifier.flag]);

  if (held.length === 0 && !alone.has(event.code)) return null;

  // The physical key rather than the character it produces: Option+E is a
  // combination, and on macOS the character it produces is an accent.
  return [...held.map((modifier) => modifier.written), key(event.code)].join(
    "+",
  );
}

/**
 * A Hotkey as it is shown: on macOS `Cmd+Shift+E` for a file that states
 * `Super+Shift+KeyE`.
 *
 * Whatever the file states, not only what this window writes — an Action can
 * arrive as a file somebody sent, with its Hotkey written however they wrote it.
 * Anything unrecognised is shown as it stands: this is a reading of a Hotkey,
 * not a judgement on one, and the backend says whether it could be claimed.
 */
export function reading(hotkey: string): string {
  const mac = navigator.userAgent.includes("Mac");

  return hotkey
    .split("+")
    .map((token) => token.trim())
    .map((token) => {
      const modifier = SPELLINGS[token.toLowerCase()];

      return modifier ? (mac ? modifier.mac : modifier.written) : key(token);
    })
    .join("+");
}

/** The key of a combination, as it is worth reading: `KeyE` is E, `Digit1` is 1. */
function key(token: string): string {
  for (const prefix of ["Key", "Digit"]) {
    if (token.startsWith(prefix) && token.length > prefix.length) {
      return token.slice(prefix.length);
    }
  }

  return token;
}

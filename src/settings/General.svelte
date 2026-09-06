<script lang="ts">
  import type { Exported } from "../lib/ipc";
  import { reading } from "../lib/hotkey";
  import { LANGUAGES } from "../lib/languages";
  import { t } from "../lib/i18n.svelte";
  import { BUTTON, FIELD } from "./style";
  import Unreadable from "./Unreadable.svelte";

  let {
    unreadable,
    language = $bindable(),
    languageFixed,
    throughThePortal,
    paletteHotkey = $bindable(),
    paletteDefault,
    recording = $bindable(),
    unclaimedHotkeys,
    autostartWanted,
    autostartProblem,
    onAutostart,
  }: {
    /** Why the settings file could not be read, when it could not be. */
    unreadable: string | null;
    language: string;
    /** The language the environment fixes, `null` where nothing is exported. */
    languageFixed: Exported | null;
    /** Whether this desktop hands out Hotkeys through a portal rather than
        letting an application claim them, which is a thing to say where they
        are set. */
    throughThePortal: boolean;
    paletteHotkey: string;
    /** What opens the Palette when nothing states otherwise, as it is read. */
    paletteDefault: string;
    /** Which Hotkey field is being recorded into, `null` for neither. */
    recording: "palette" | "action" | null;
    /** The Hotkeys stated by an Action and not answered to, in the backend's
        own sentences. */
    unclaimedHotkeys: string[];
    /** Whether Demysto is in the login items, as the system last answered. */
    autostartWanted: boolean;
    /** What the system said the last time it would not change them. */
    autostartProblem: string | null;
    onAutostart: (wanted: boolean) => void;
  } = $props();

  /** Takes the Palette back to the Hotkey Demysto comes with. */
  function unbindPalette() {
    paletteHotkey = "";
    recording = null;
  }
</script>

{#if unreadable}
  <Unreadable said={unreadable} />
{:else}
  <section class="flex flex-col gap-3">
    <h2 class="text-xs font-semibold tracking-wide uppercase opacity-50">
      {t("settings-language")}
    </h2>

    <label class="flex flex-col gap-1">
      <span class="text-xs opacity-60">{t("settings-language-field")}</span>
      <select bind:value={language} class="{FIELD} max-w-64">
        <option value="">{t("settings-language-follows-system")}</option>
        {#each LANGUAGES as offered (offered.tag)}
          <option value={offered.tag}>{offered.name}</option>
        {/each}
      </select>
    </label>

    <span class="text-xs opacity-50">{t("settings-language-detail")}</span>

    {#if languageFixed}
      <!-- Said where the field is, for the reason a key found in a variable
           is: without it, somebody choosing a language here and watching
           nothing change has no way to learn why. -->
      <p class="text-xs opacity-50">
        {t("settings-language-from-environment", {
          variable: languageFixed.variable,
          value: languageFixed.value,
        })}
      </p>
    {/if}
  </section>
{/if}

<section class="flex flex-col gap-3">
  <h2 class="text-xs font-semibold tracking-wide uppercase opacity-50">
    {t("settings-hotkeys")}
  </h2>

  {#if throughThePortal}
    <!-- The half of what Wayland costs that is about this field. The other
         half — that a Capture there reads only the clipboard — is said above
         the tabs, because it is an answer to "why did pressing it do that?"
         wherever in this window somebody is standing. See ADR-0003. -->
    <p class="text-xs opacity-50">{t("settings-wayland-hotkeys")}</p>
  {/if}

  {#if !unreadable}
    <!-- A row of four sentences, and until ticket 26 a screen reader was
         given two buttons and none of them: what the row is for, what is
         bound now, that a recording is under way and what may be pressed
         were bare runs of text, which WebKitGTK keeps in no tree — the
         same gap ticket 18 found in the Palette's header, in a second
         place.

         So the combination is the *value* of this row rather than a
         sentence beside it: the caption names it, and reaching it reads
         both, in the order somebody would ask. `status` is what keeps it
         in the tree at all, and here it speaks as well — unlike the
         Palette's caption, what changes these words is the user pressing
         Record, so there is somebody listening when they change. The one
         change nobody asks for — the settings arriving and putting the
         user's own combination where the built-in one was — happens at
         startup, while this window is loaded and hidden, which is where
         18's caption changes too and where an announcement reaches
         nobody.

         The rule is what may be pressed, so it is on the button that asks
         for a combination, and the caption is on both buttons: the value
         is not focusable, and a Tab that arrives at "Record" would
         otherwise arrive at a word with nothing saying which of the two
         rows it belongs to. The rule needed no role of its own — it is a
         paragraph now rather than a run of text inside a row, and a
         paragraph is kept.

         Nothing here draws anything: the same words stay in the same
         places at the same size. -->
    <div class="flex flex-col gap-1">
      <span id="palette-hotkey" class="text-xs opacity-60">
        {t("settings-palette-hotkey")}
      </span>
      <div class="flex items-center gap-2">
        <span
          role="status"
          aria-labelledby="palette-hotkey"
          class="{FIELD} flex-1 truncate {paletteHotkey ||
          recording === 'palette'
            ? ''
            : 'opacity-40'}"
        >
          {#if recording === "palette"}
            {t("settings-hotkey-recording")}
          {:else if paletteHotkey}
            {reading(paletteHotkey)}
          {:else}
            {t("settings-hotkey-default", {
              hotkey: reading(paletteDefault),
            })}
          {/if}
        </span>

        <button
          type="button"
          class={BUTTON}
          aria-describedby="palette-hotkey palette-hotkey-rule"
          disabled={recording !== null && recording !== "palette"}
          onclick={() =>
            (recording = recording === "palette" ? null : "palette")}
        >
          {recording === "palette"
            ? t("settings-hotkey-cancel")
            : t("settings-hotkey-record")}
        </button>

        <button
          type="button"
          class={BUTTON}
          aria-describedby="palette-hotkey"
          disabled={!paletteHotkey}
          onclick={unbindPalette}
        >
          {t("settings-hotkey-clear")}
        </button>
      </div>
      <p id="palette-hotkey-rule" class="text-xs opacity-50">
        {t("settings-hotkey-rule")}
        {t("settings-palette-hotkey-detail")}
      </p>
    </div>
  {/if}

  {#each unclaimedHotkeys as said (said)}
    <p class="text-xs text-red-600 dark:text-red-400">{said}</p>
  {/each}
</section>

<section class="flex flex-col gap-3">
  <h2 class="text-xs font-semibold tracking-wide uppercase opacity-50">
    {t("settings-autostart")}
  </h2>

  <p class="text-xs opacity-50">{t("settings-autostart-detail")}</p>

  <label class="flex items-center gap-2 text-sm">
    <input
      type="checkbox"
      checked={autostartWanted}
      onchange={(event) => onAutostart(event.currentTarget.checked)}
    />
    {t("settings-autostart-choice")}
  </label>

  {#if autostartProblem}
    <p class="text-xs text-red-600 dark:text-red-400">{autostartProblem}</p>
  {/if}
</section>

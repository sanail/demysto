<script lang="ts">
  import { onDestroy } from "svelte";
  import type { Exported } from "../lib/ipc";
  import { LANGUAGES } from "../lib/languages";
  import { t } from "../lib/i18n.svelte";
  import { FIELD } from "./style";
  import HotkeyField from "./HotkeyField.svelte";
  import Unreadable from "./Unreadable.svelte";

  let {
    unreadable,
    language = $bindable(),
    languageFixed,
    throughThePortal,
    screenAreaOff,
    paletteHotkey = $bindable(),
    screenAreaHotkey = $bindable(),
    paletteDefault,
    screenAreaDefault,
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
    /** Whether no area of the screen can be had here, which disables its
        field. */
    screenAreaOff: boolean;
    /** `null` for the default, empty for none. */
    paletteHotkey: string | null;
    /** Read like the Palette's. */
    screenAreaHotkey: string | null;
    /** What opens the Palette when nothing states otherwise, as it is read. */
    paletteDefault: string;
    /** The same for the screen area. */
    screenAreaDefault: string;
    /** Which Hotkey field is being recorded into, `null` for neither. */
    recording: "palette" | "screen_area" | "action" | null;
    /** The Hotkeys stated by an Action and not answered to, in the backend's
        own sentences. */
    unclaimedHotkeys: string[];
    /** Whether Demysto is in the login items, as the system last answered. */
    autostartWanted: boolean;
    /** What the system said the last time it would not change them. */
    autostartProblem: string | null;
    /** Changes the login items, answering whether the system did it. */
    onAutostart: (wanted: boolean) => Promise<boolean>;
  } = $props();

  /** How long the box says it took effect, which is the Save button's own. */
  const ACKNOWLEDGED = 1600;

  let autostartDone = $state(false);
  let acknowledging: ReturnType<typeof setTimeout> | null = null;

  /**
   * Ticks the box and says so.
   *
   * This one field acts as it is ticked rather than waiting for Save, and until
   * it said as much the only way to learn that was the sentence under it. The
   * system's own answer is what is acknowledged, not the click: a refusal
   * leaves the box where it was and puts its reason below instead.
   */
  async function autostart(wanted: boolean) {
    if (acknowledging) clearTimeout(acknowledging);
    autostartDone = false;

    if (!(await onAutostart(wanted))) return;

    autostartDone = true;
    acknowledging = setTimeout(() => (autostartDone = false), ACKNOWLEDGED);
  }

  onDestroy(() => {
    if (acknowledging) clearTimeout(acknowledging);
  });
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
    <HotkeyField
      which="palette"
      caption={t("settings-palette-hotkey")}
      bind:bound={paletteHotkey}
      fallback={paletteDefault}
      bind:recording
    />
    <HotkeyField
      which="screen_area"
      caption={t("settings-screen-area-hotkey")}
      bind:bound={screenAreaHotkey}
      fallback={screenAreaDefault}
      bind:recording
      off={screenAreaOff}
    />
    {#if screenAreaOff}
      <p id="screen_area-unavailable" class="text-xs opacity-50">
        {t("settings-screen-area-unavailable")}
      </p>
    {/if}
    <p id="hotkey-rule" class="text-xs opacity-50">
      {t("settings-palette-hotkey-rule")}
    </p>
  {/if}

  {#each unclaimedHotkeys as said (said)}
    <p class="text-xs text-red-600 dark:text-red-400">{said}</p>
  {/each}
</section>

<section class="flex flex-col gap-3">
  <h2 class="text-xs font-semibold tracking-wide uppercase opacity-50">
    {t("settings-autostart")}
  </h2>

  <div class="flex items-center gap-2">
    <label class="flex items-center gap-2 text-sm">
      <input
        type="checkbox"
        checked={autostartWanted}
        onchange={(event) => autostart(event.currentTarget.checked)}
      />
      {t("settings-autostart-choice")}
    </label>

    <!-- Drawn empty rather than left out, so that the live region is already
         there when it has something to say. -->
    <span role="status" class="text-xs opacity-50">
      {autostartDone ? t("settings-autostart-changed") : ""}
    </span>
  </div>

  <!-- Under the box now that the box says for itself that it took effect:
       what is left is a footnote about whose list this is. -->
  <p class="text-xs opacity-50">{t("settings-autostart-detail")}</p>

  {#if autostartProblem}
    <p class="text-xs text-red-600 dark:text-red-400">{autostartProblem}</p>
  {/if}
</section>

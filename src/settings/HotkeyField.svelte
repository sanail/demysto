<script lang="ts">
  import { reading } from "../lib/hotkey";
  import { t } from "../lib/i18n.svelte";
  import { BUTTON, FIELD } from "./style";

  /**
   * One of the two Hotkeys, with its record, clear and restore buttons. The
   * recording itself is the window's, which owns the keypresses.
   */
  let {
    which,
    caption,
    bound = $bindable(),
    fallback,
    recording = $bindable(),
    off = false,
  }: {
    which: "palette" | "screen_area";
    caption: string;
    /** `null` for the default, empty for none. */
    bound: string | null;
    /** What answers when nothing states otherwise, as it is read. */
    fallback: string;
    /** Which field is being recorded into, `null` for none. */
    recording: "palette" | "screen_area" | "action" | null;
    /** Whether this desktop has no use for the Hotkey at all. */
    off?: boolean;
  } = $props();

  function same(one: string, other: string): boolean {
    return reading(one).toLowerCase() === reading(other).toLowerCase();
  }
</script>

<div class="flex flex-col gap-1">
  <span id="{which}-hotkey" class="text-xs opacity-60">{caption}</span>
  <div class="flex items-center gap-2">
    <!-- The combination is the row's value rather than a sentence beside it,
         so a screen reader reads caption and value together; `status` keeps
         it in WebKitGTK's tree. The rule is on the Record button. -->
    <span
      role="status"
      aria-labelledby="{which}-hotkey"
      class="{FIELD} flex-1 truncate {bound || recording === which
        ? ''
        : 'opacity-40'}"
    >
      {#if recording === which}
        {t("settings-hotkey-recording")}
      {:else if bound === null}
        {t("settings-hotkey-default", { hotkey: reading(fallback) })}
      {:else if bound}
        {reading(bound)}
      {:else}
        {t("settings-hotkey-cleared")}
      {/if}
    </span>

    <button
      type="button"
      class={BUTTON}
      aria-describedby="{which}-hotkey {off
        ? `${which}-unavailable`
        : 'hotkey-rule'}"
      disabled={off || (recording !== null && recording !== which)}
      onclick={() => (recording = recording === which ? null : which)}
    >
      {recording === which
        ? t("settings-hotkey-cancel")
        : t("settings-hotkey-record")}
    </button>

    {#if bound !== ""}
      <button
        type="button"
        class={BUTTON}
        aria-label={t("settings-hotkey-clear")}
        title={t("settings-hotkey-clear")}
        aria-describedby="{which}-hotkey"
        disabled={off}
        onclick={() => {
          bound = "";
          recording = null;
        }}
      >
        ×
      </button>
    {/if}
    <!-- Stated by hand as the default itself, it is the default already. -->
    {#if bound !== null && !same(bound, fallback)}
      <button
        type="button"
        class={BUTTON}
        aria-label={t("settings-hotkey-restore")}
        title={t("settings-hotkey-restore")}
        aria-describedby="{which}-hotkey"
        disabled={off}
        onclick={() => {
          bound = null;
          recording = null;
        }}
      >
        ↺
      </button>
    {/if}
  </div>
</div>

<script lang="ts">
  import { installUpdate, lookForUpdate, openLogs } from "../lib/ipc";
  import { t } from "../lib/i18n.svelte";
  import { saidBy, sending } from "../lib/sending";
  import { BUTTON } from "./style";

  let {
    version,
    where,
    newer = $bindable(),
  }: {
    /** The version this is, which is the half of an update question nobody
        else answers. */
    version: string;
    /** The directory the settings file is in. */
    where: string;
    /** The newer version there is, `null` where there is none to be had. */
    newer: string | null;
  } = $props();

  /** What went wrong opening the log folder. */
  let logsProblem = $state<string | null>(null);
  /**
   * Whether a check has been made while this window has been open.
   *
   * What separates "up to date" from "not asked yet": the window opens knowing
   * only what the check on the way up left behind, and that answer is silence
   * when the machine was offline for it.
   */
  let asked = $state(false);
  let checking = $state(false);
  let installing = $state(false);
  let updateProblem = $state<string | null>(null);

  /** Opens the folder the logs are written in, so a bug report can carry them. */
  async function showLogs() {
    logsProblem = await sending(openLogs);
  }

  /** Asks the manifest whether there is a newer Demysto than this one. */
  async function checkForUpdate() {
    checking = true;
    updateProblem = null;

    try {
      newer = await lookForUpdate();
      asked = true;
    } catch (error) {
      updateProblem = saidBy(error);
    } finally {
      checking = false;
    }
  }

  /**
   * Takes the update on offer.
   *
   * Nothing follows a success, and nothing can: the process is replaced by the
   * version it installed. The state is left saying "installing" for exactly
   * that reason — the only way back from here is a failure.
   */
  async function install() {
    installing = true;
    updateProblem = null;

    try {
      await installUpdate();
    } catch (error) {
      updateProblem = saidBy(error);
      installing = false;
    }
  }
</script>

<!-- Written out rather than truncated with the full text in a title, which is
     where this path used to live: somebody wants it in order to open the file
     by hand or to put it in a bug report, and both want text that can be
     selected and read aloud.

     The folder and not the file, because the folder is what the backend
     reports and what is actually wanted: the settings file is in it, and so
     are the Actions. -->
<section class="flex flex-col gap-1">
  <h2 class="text-xs font-semibold tracking-wide uppercase opacity-50">
    {t("settings-folder")}
  </h2>

  <p class="text-xs break-all opacity-60">{where}</p>
</section>

<section class="flex flex-col gap-3">
  <h2 class="text-xs font-semibold tracking-wide uppercase opacity-50">
    {t("settings-logs")}
  </h2>

  <p class="text-xs opacity-50">{t("settings-logs-detail")}</p>

  <div>
    <button type="button" class={BUTTON} onclick={showLogs}>
      {t("settings-open-logs")}
    </button>
  </div>

  {#if logsProblem}
    <p class="text-xs text-red-600 dark:text-red-400">{logsProblem}</p>
  {/if}
</section>

<section class="flex flex-col gap-3">
  <h2 class="text-xs font-semibold tracking-wide uppercase opacity-50">
    {t("settings-updates")}
  </h2>

  <p class="text-xs opacity-50">{t("settings-updates-detail")}</p>

  <p class="text-xs opacity-60">{t("settings-version", { version })}</p>

  <div class="flex items-center gap-2">
    <button
      type="button"
      class={BUTTON}
      disabled={checking || installing}
      onclick={checkForUpdate}
    >
      {checking ? t("settings-checking") : t("settings-check-for-update")}
    </button>

    {#if newer}
      <button
        type="button"
        class={BUTTON}
        disabled={installing}
        onclick={install}
      >
        {installing
          ? t("settings-installing")
          : t("settings-install-update")}
      </button>
    {/if}
  </div>

  {#if newer}
    <p class="text-xs">{t("settings-update-found", { version: newer })}</p>
  {:else if asked}
    <p class="text-xs opacity-50">{t("settings-up-to-date")}</p>
  {/if}

  {#if updateProblem}
    <p class="text-xs text-red-600 dark:text-red-400">{updateProblem}</p>
  {/if}
</section>

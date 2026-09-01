<script lang="ts">
  import { onMount } from "svelte";
  import {
    accessibilityAskedFor,
    autostart,
    dismiss,
    hotkeys as allowed,
    openAccessibility,
    presets as offeredPresets,
    providerModels,
    saveSettings,
    setAutostart,
    settings as configured,
    status,
    verifyProvider,
    type Capturing,
    type ConfiguredProvider,
    type Edit,
    type Exported,
    type Preset,
    type ProviderEdit,
    type Settings,
  } from "../lib/ipc";
  import { reading } from "../lib/hotkey";
  import { LANGUAGES } from "../lib/languages";
  import { spokenTag, t } from "../lib/i18n.svelte";
  import { saidBy, sending } from "../lib/sending";

  /**
   * The flow, in the order the spec fixes it: confirm the language, configure a
   * Provider and prove its key works, walk to the Accessibility permission,
   * answer the autostart question, and finish on the Hotkey.
   */
  type Step = "language" | "provider" | "accessibility" | "autostart" | "done";

  const ORDER: Step[] = [
    "language",
    "provider",
    "accessibility",
    "autostart",
    "done",
  ];

  /**
   * Whether this desktop gates a Capture behind a permission, which is the one
   * step not every platform has. `null` until the backend has answered, and the
   * step is left out until it says yes: only macOS has such a pane, and walking
   * a Windows user to one their system has never heard of would teach them the
   * tool does not know where it is running.
   */
  let asksForAccessibility = $state(false);

  /** The steps this desktop has, in the order the spec fixes them. */
  const steps = $derived(
    ORDER.filter((step) => step !== "accessibility" || asksForAccessibility),
  );
  let at = $state(0);
  const step = $derived(steps[at]);

  /**
   * The settings as the file holds them, which every save is built back out of:
   * the flow writes the file three or four times over, and each write is the
   * whole of it (see `Edit`). Anything the flow does not ask about — a Hotkey
   * or a Provider from an installation that has been here before — travels
   * through untouched rather than being dropped by the step that never
   * mentioned it.
   */
  let saved = $state<Settings | null>(null);
  let presets = $state<Preset[]>([]);
  /** What opens the Palette, which is what the last step invites a press of. */
  let paletteDefault = $state("");

  /**
   * The language the flow is being read in, which is the one it offers for
   * confirmation.
   */
  let language = $state(spokenTag());
  /**
   * And the one it was found in, so that confirming it writes nothing.
   *
   * Confirming means "whatever decided this was right", not "write this down":
   * a file stating the language it was already following would stop following
   * the operating system, and would bake in a `DEMYSTO_LANGUAGE` exported for
   * one launch (user story 58). Choosing another language is the change, and
   * that is what gets written.
   */
  let detected = $state(spokenTag());
  /**
   * The language the environment fixes, `null` where nothing is exported. The
   * field is still offered where it is set — what is written is what will be
   * spoken the moment the variable is not — and the sentence beside it says so,
   * as Settings does.
   */
  let languageFixed = $state<Exported | null>(null);
  /**
   * The sentence a desktop that will not let Demysto read a Selection is owed,
   * `null` everywhere else (user story 56).
   *
   * The last step invites a press of the Hotkey over selected text, and on
   * Wayland that is the one thing it cannot do (ADR-0003): the invitation would
   * be teaching a new user a gesture that produces nothing. So the same
   * sentence the Palette and Settings say is said here, before they ever reach
   * either of them.
   */
  let clipboardOnly = $state<string | null>(null);

  /**
   * What a Capture on this desktop cannot do, where there is such a thing.
   *
   * Named for the reading rather than for the sentence, unlike Settings' own
   * `said`: this window already has a `said` — what the Provider answered.
   */
  function cannotRead(capturing: Capturing): string | null {
    return capturing.reads === "clipboard_only"
      ? t("capture-clipboard-only")
      : null;
  }

  /**
   * The Provider being configured, as this window has it. One rather than the
   * list Settings edits: the flow's job is the first Provider, and the second
   * is what Settings is for.
   */
  let name = $state("");
  let preset = $state("");
  let baseUrl = $state("");
  /**
   * The key as it was typed, which is the only way one gets in — and out of
   * here it goes nowhere but the backend, per ADR-0002.
   */
  let key = $state("");
  let model = $state("");
  /** What the Provider said it offers, once somebody asked it. */
  let offered = $state<string[] | null>(null);
  /** What this Provider was called in the file, once the flow has written it. */
  let was = $state<string | null>(null);

  /** Whether a Provider is being asked something now. */
  let asking = $state(false);
  /** What it last said, and whether that was good news. */
  let said = $state<{ well: boolean; message: string } | null>(null);
  /**
   * Whether this key has been put to its Provider and accepted.
   *
   * The flow does not go past the Provider until it has: a key that turns out
   * to be wrong at the first Run is exactly the first impression this ticket
   * exists to prevent (user story 42).
   */
  let verified = $state(false);

  let autostartWanted = $state(false);
  /** What stopped the login items being changed, in the backend's own words. */
  let autostartProblem = $state<string | null>(null);
  /** And what stopped the Accessibility pane being opened. */
  let accessibilityProblem = $state<string | null>(null);
  /** And what stopped a save, which is the one failure that holds the flow up. */
  let problem = $state<string | null>(null);
  let saving = $state(false);

  const FIELD =
    "w-full rounded border border-neutral-300 bg-transparent px-2 py-1 text-sm " +
    "outline-none focus:border-neutral-500 dark:border-neutral-700 " +
    "dark:focus:border-neutral-500";

  const BUTTON =
    "cursor-pointer rounded border border-neutral-300 px-2 py-1 text-xs " +
    "hover:bg-neutral-100 disabled:cursor-default disabled:opacity-40 " +
    "disabled:hover:bg-transparent dark:border-neutral-700 " +
    "dark:hover:bg-neutral-800 dark:disabled:hover:bg-transparent";

  const PRIMARY =
    "cursor-pointer rounded bg-neutral-900 px-3 py-1.5 text-xs font-medium " +
    "text-white hover:bg-neutral-700 disabled:cursor-default " +
    "disabled:opacity-40 disabled:hover:bg-neutral-900 dark:bg-neutral-100 " +
    "dark:text-neutral-900 dark:hover:bg-neutral-300 " +
    "dark:disabled:hover:bg-neutral-100";

  onMount(async () => {
    presets = await offeredPresets();
    paletteDefault = (await allowed()).palette_default;
    autostartWanted = await autostart();
    const reported = await status();
    languageFixed = reported.language_env;
    clipboardOnly = cannotRead(reported.capturing);

    asksForAccessibility = await accessibilityAskedFor();

    try {
      show(await configured());
    } catch (error) {
      // A file nobody can read is not one the flow can write into, and the
      // backend does not offer the flow over one — so this is a file that
      // broke between the two. Said here, where the save that will fail is
      // about to be asked for.
      problem = saidBy(error);
    }
  });

  function show(settings: Settings) {
    saved = settings;
    language = settings.language ?? spokenTag();
    detected = language;
  }

  /** The Hotkey the last step invites a press of, as the user reads it. */
  const hotkey = $derived(reading(saved?.palette_hotkey ?? paletteDefault));

  /** The Provider as it will be written. */
  function drafted(): ProviderEdit {
    return {
      was,
      name,
      base_url: baseUrl,
      preset,
      api_key_env: null,
      // Typing nothing leaves whatever the file holds — which on a first run is
      // nothing, and for a service with no keys is the right answer anyway
      // (ADR-0006). A key found in the environment is resolved by the backend
      // and needs no field here; Verify is what proves either way.
      api_key: key.trim() === "" ? { action: "keep" } : { action: "set", key },
      models: model.trim() === "" ? [] : [{ id: model, vision: false }],
    };
  }

  /**
   * The whole of the settings as the flow would now have them.
   *
   * Every Provider the file already holds travels through with its key left
   * alone; the one being configured replaces the copy of itself written by an
   * earlier step, so that going back and forward does not configure it twice.
   */
  function edit(): Edit {
    const others = (saved?.providers ?? [])
      .filter((provider) => provider.name !== was)
      .map(kept);

    const configuring = name.trim() !== "";

    return {
      providers: configuring ? [...others, drafted()] : others,
      default_model:
        configuring && model.trim() !== ""
          ? `${name}/${model}`
          : (saved?.default_model ?? null),
      default_vision_model: saved?.default_vision_model ?? null,
      palette_hotkey: saved?.palette_hotkey ?? null,
      large_selection: saved?.large_selection ?? null,
      // Only a language the user actually chose. Leaving the one they were
      // shown alone leaves the file saying nothing about it, which is Demysto
      // going on following the operating system.
      language: language === detected ? (saved?.language ?? null) : language,
    };
  }

  /** A Provider the flow is not touching, as an edit that changes nothing. */
  function kept(provider: ConfiguredProvider): ProviderEdit {
    return {
      was: provider.name,
      name: provider.name,
      base_url: provider.base_url,
      preset: provider.preset,
      api_key_env: provider.api_key_env,
      api_key: { action: "keep" },
      models: provider.models.map((model) => ({ ...model })),
    };
  }

  /** Fills in what a preset knows, so that picking one is most of the step. */
  function picked() {
    changed();
    offered = null;

    const chosen = presets.find((it) => it.name === preset);
    if (chosen && name.trim() === "") name = chosen.name;
  }

  /**
   * Anything about the Provider having changed puts the proof back where it
   * was: a key verified against one base URL says nothing about another.
   */
  function changed() {
    verified = false;
    said = null;
  }

  /** Asks the Provider what it offers, so that nobody types an identifier from
      memory (user story 34). */
  async function askForModels() {
    asking = true;
    said = null;

    try {
      offered = await providerModels(drafted());
      if (offered.length === 0) {
        said = { well: false, message: t("settings-provider-offers-nothing") };
      } else if (model.trim() === "") {
        model = offered[0];
      }
    } catch (error) {
      said = { well: false, message: saidBy(error) };
    } finally {
      asking = false;
    }
  }

  /** Puts the smallest real request to the Provider, which is what lets the
      flow go on (ADR-0008). */
  async function verify() {
    asking = true;
    said = null;

    try {
      await verifyProvider(drafted(), model);
      verified = true;
      said = { well: true, message: t("settings-provider-answered", { model }) };
    } catch (error) {
      verified = false;
      said = { well: false, message: saidBy(error) };
    } finally {
      asking = false;
    }
  }

  /** Writes the settings as the flow now has them, and shows what came back. */
  async function save(): Promise<boolean> {
    saving = true;
    problem = null;

    try {
      show(await saveSettings(edit()));
      if (name.trim() !== "") was = name;

      return true;
    } catch (error) {
      problem = saidBy(error);

      return false;
    } finally {
      saving = false;
    }
  }

  /**
   * Answers the login items question, and reports a system that would not have
   * it either way.
   *
   * Acted on as it is answered rather than at the end of the flow: the checkbox
   * is the choice, and one that took effect somewhere else would be a choice
   * the user cannot see they have made (user story 52).
   */
  async function autostartIs(wanted: boolean) {
    autostartWanted = wanted;
    autostartProblem = await sending(() => setAutostart(wanted));

    // What the system says it did, rather than what it was asked for: a
    // refusal leaves the box where it was rather than lying about it.
    if (autostartProblem !== null) autostartWanted = await autostart();
  }

  async function showAccessibility() {
    accessibilityProblem = await sending(openAccessibility);
  }

  /**
   * Whether the step on screen has been answered well enough to leave.
   *
   * The Provider step wants all three: a name to configure it under, a Model to
   * nominate, and the Provider's own word that the key works. The verification
   * alone is not enough, and not for a hypothetical reason — it puts its request
   * to an endpoint rather than to a configured Provider, so it answers just as
   * happily for a draft with no name at all. That draft is not written by the
   * save that follows, and the flow would walk on to its last step having
   * configured nothing.
   */
  const settled = $derived(
    step !== "provider" ||
      (verified && name.trim() !== "" && model.trim() !== ""),
  );

  async function forward() {
    // The language and the Provider are written as the step that collected them
    // is left. The language is written twice over — once here and again with
    // the Provider — because a flow abandoned half-way should still leave
    // Demysto speaking what was confirmed in it.
    if (step === "language" || step === "provider") {
      if (!(await save())) return;
    }

    if (at + 1 < steps.length) {
      at += 1;
      return;
    }

    // The end of the flow is the window going away, and the backend records it
    // as over when it does — however it goes; see the shell's `welcome`.
    dismiss();
  }

  function back() {
    problem = null;
    if (at > 0) at -= 1;
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key !== "Escape") return;

    event.preventDefault();
    dismiss();
  }
</script>

<svelte:window onkeydown={onKeydown} />

<main
  class="flex h-screen flex-col gap-5 bg-white p-6 font-sans text-neutral-900
         dark:bg-neutral-900 dark:text-neutral-100"
>
  <header class="flex items-baseline justify-between gap-3">
    <h1 class="text-sm font-semibold tracking-tight">{t("welcome-title")}</h1>
    <span class="text-xs opacity-40">
      {t("welcome-step", { at: at + 1, total: steps.length })}
    </span>
  </header>

  <div class="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto">
    {#if step === "language"}
      <section class="flex flex-col gap-3">
        <h2 class="text-sm font-medium">{t("welcome-language-title")}</h2>
        <p class="text-sm opacity-60">{t("welcome-language-detail")}</p>

        <label class="flex flex-col gap-1">
          <span class="text-xs opacity-60">{t("settings-language-field")}</span>
          <select bind:value={language} class={FIELD}>
            {#each LANGUAGES as offered (offered.tag)}
              <option value={offered.tag}>{offered.name}</option>
            {/each}
          </select>
        </label>

        {#if languageFixed}
          <p class="text-xs opacity-50">
            {t("settings-language-from-environment", {
              variable: languageFixed.variable,
              value: languageFixed.value,
            })}
          </p>
        {/if}
      </section>
    {:else if step === "provider"}
      <section class="flex flex-col gap-3">
        <h2 class="text-sm font-medium">{t("welcome-provider-title")}</h2>
        <p class="text-sm opacity-60">{t("welcome-provider-detail")}</p>

        <div class="grid grid-cols-2 gap-3">
          <label class="flex flex-col gap-1">
            <span class="text-xs opacity-60">
              {t("settings-provider-service")}
            </span>
            <select bind:value={preset} onchange={picked} class={FIELD}>
              <option value="">{t("settings-provider-no-preset")}</option>
              {#each presets as offered (offered.name)}
                <option value={offered.name}>
                  {offered.needs_key
                    ? offered.name
                    : t("settings-provider-preset-keyless", {
                        preset: offered.name,
                      })}
                </option>
              {/each}
            </select>
          </label>

          <label class="flex flex-col gap-1">
            <span class="text-xs opacity-60">{t("settings-provider-name")}</span>
            <input
              bind:value={name}
              oninput={changed}
              class={FIELD}
              placeholder={t("settings-provider-name-example")}
            />
          </label>

          <label class="col-span-2 flex flex-col gap-1">
            <span class="text-xs opacity-60">
              {preset === ""
                ? t("settings-provider-base-url")
                : t("settings-provider-base-url-from-preset")}
            </span>
            <input
              bind:value={baseUrl}
              oninput={changed}
              class={FIELD}
              placeholder={presets.find((it) => it.name === preset)?.base_url ??
                t("settings-provider-base-url-example")}
            />
          </label>

          <label class="col-span-2 flex flex-col gap-1">
            <span class="text-xs opacity-60">{t("settings-provider-key")}</span>
            <input
              type="password"
              bind:value={key}
              oninput={changed}
              class={FIELD}
              placeholder={t("settings-key-missing")}
            />
          </label>
        </div>

        <div class="flex items-end gap-2">
          <label class="flex flex-1 flex-col gap-1">
            <span class="text-xs opacity-60">{t("welcome-provider-model")}</span>
            {#if offered && offered.length > 0}
              <select bind:value={model} onchange={changed} class={FIELD}>
                {#each offered as id (id)}
                  <option value={id}>{id}</option>
                {/each}
              </select>
            {:else}
              <input bind:value={model} oninput={changed} class={FIELD} />
            {/if}
          </label>

          <button
            type="button"
            class={BUTTON}
            disabled={asking}
            onclick={askForModels}
          >
            {t("settings-fetch-models")}
          </button>

          <button
            type="button"
            class={BUTTON}
            disabled={asking || name.trim() === "" || model.trim() === ""}
            onclick={verify}
          >
            {t("settings-verify-key")}
          </button>
        </div>

        {#if asking}
          <p class="text-xs opacity-50">{t("settings-asking-provider")}</p>
        {:else if said}
          <p
            class="text-xs {said.well
              ? 'text-green-700 dark:text-green-400'
              : 'text-red-600 dark:text-red-400'}"
          >
            {said.message}
          </p>
        {:else}
          <p class="text-xs opacity-50">{t("welcome-provider-verify-first")}</p>
        {/if}
      </section>
    {:else if step === "accessibility"}
      <section class="flex flex-col gap-3">
        <h2 class="text-sm font-medium">{t("welcome-accessibility-title")}</h2>
        <!-- Not the sentence the rest of Demysto says when a Capture is
             refused: that one reports a refusal, and nothing here knows
             whether the permission is granted or not. This says what the
             permission is for and where it is granted. -->
        <p class="text-sm opacity-60">{t("welcome-accessibility-detail")}</p>

        <div>
          <button type="button" class={BUTTON} onclick={showAccessibility}>
            {t("welcome-open-accessibility")}
          </button>
        </div>

        {#if accessibilityProblem}
          <p class="text-xs text-red-600 dark:text-red-400">
            {accessibilityProblem}
          </p>
        {/if}

        <p class="text-xs opacity-50">{t("welcome-accessibility-later")}</p>
      </section>
    {:else if step === "autostart"}
      <section class="flex flex-col gap-3">
        <h2 class="text-sm font-medium">{t("welcome-autostart-title")}</h2>
        <p class="text-sm opacity-60">{t("welcome-autostart-detail")}</p>

        <label class="flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={autostartWanted}
            onchange={(event) => autostartIs(event.currentTarget.checked)}
          />
          {t("welcome-autostart-choice")}
        </label>

        {#if autostartProblem}
          <p class="text-xs text-red-600 dark:text-red-400">
            {autostartProblem}
          </p>
        {/if}
      </section>
    {:else}
      <section class="flex flex-col gap-3">
        <h2 class="text-sm font-medium">{t("welcome-done-title")}</h2>
        <!-- The invitation is adjusted rather than contradicted: on Wayland
             there is nothing to be gained by telling somebody to select text
             first and explaining underneath that it will not be read. -->
        <p class="text-sm opacity-60">
          {clipboardOnly
            ? t("welcome-done-clipboard", { hotkey })
            : t("welcome-done-detail", { hotkey })}
        </p>
        {#if clipboardOnly}
          <p class="text-sm opacity-60">{clipboardOnly}</p>
        {/if}
        <p class="text-sm opacity-60">{t("welcome-done-tray")}</p>
      </section>
    {/if}

    {#if problem}
      <p class="text-sm text-red-600 dark:text-red-400">{problem}</p>
    {/if}
  </div>

  <footer class="flex items-center justify-between gap-3">
    <button
      type="button"
      class="{BUTTON} {at === 0 ? 'invisible' : ''}"
      onclick={back}
    >
      {t("welcome-back")}
    </button>

    <button
      type="button"
      class={PRIMARY}
      disabled={saving || !settled}
      onclick={forward}
    >
      {step === "done" ? t("welcome-finish") : t("welcome-continue")}
    </button>
  </footer>
</main>

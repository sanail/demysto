<script lang="ts">
  import { onDestroy, onMount, tick } from "svelte";
  import {
    catalogue as catalogued,
    deleteAction,
    dismiss,
    hotkeys as allowed,
    onProviderWanted,
    openLogs,
    presets as offeredPresets,
    providerModels,
    saveAction,
    saveSettings,
    settings as configured,
    status,
    verifyProvider,
    type ActionEdit,
    type ActionStanding,
    type Capturing,
    type Catalogue,
    type ConfiguredModel,
    type ConfiguredProvider,
    type DefinedAction,
    type KeyEdit,
    type KeyStanding,
    type Exported,
    type Preset,
    type ProviderEdit,
    type Settings,
  } from "../lib/ipc";
  import { combination, reading } from "../lib/hotkey";
  import { LANGUAGES } from "../lib/languages";
  import { spokenTag, t } from "../lib/i18n.svelte";
  import { saidBy, sending } from "../lib/sending";
  import type { UnlistenFn } from "@tauri-apps/api/event";

  /** How long the window says a save landed. */
  const ACKNOWLEDGED = 1600;

  /** What a Capture on this desktop cannot do, where there is such a thing. */
  function said(capturing: Capturing): string | null {
    return capturing.reads === "clipboard_only"
      ? t("capture-clipboard-only")
      : null;
  }

  /**
   * One Provider as this window has it: what will be written, plus what is only
   * ever on screen — where its key already is, what it answered when it was
   * last asked something, and whether it is being asked something now.
   */
  type Draft = {
    was: string | null;
    name: string;
    base_url: string;
    preset: string;
    api_key_env: string;
    models: ConfiguredModel[];
    /**
     * Where the key the file holds is. Never the key: this window is the one
     * ADR-0002 promises the key does not enter, and a field showing it would
     * be that promise broken for the sake of showing somebody their own
     * secret back.
     */
    standing: KeyStanding;
    /** What has been typed into the key field, which is the only way one gets in. */
    typed: string;
    /** Whether the file's own key is to be taken out on the next save. */
    forgetting: boolean;
    /** What this Provider said it offers, once somebody asked it. */
    offered: string[] | null;
    /** Which of its Models a verification puts its request to. */
    trying: string;
    /** Whether it is being asked something now. */
    asking: boolean;
    /** What it last said, and whether that was good news. */
    said: { well: boolean; message: string } | null;
  };

  /**
   * One Action being edited, and where its definition stood before the editing
   * began — `null` for one being written, which stands nowhere yet.
   *
   * One at a time, and not a draft each: an Action is a file of its own and is
   * saved on its own, so there is never more than one unsaved.
   */
  type Editing = { draft: ActionEdit; standing: ActionStanding | null };

  let drafts = $state<Draft[]>([]);
  let defaultModel = $state("");
  let defaultVisionModel = $state("");
  let presets = $state<Preset[]>([]);
  let where = $state("");

  /** The Actions as the directory holds them, and what could not be read. */
  let actions = $state<DefinedAction[]>([]);
  let unreadableActions = $state<string[]>([]);
  /**
   * The Hotkeys stated by an Action and not answered to, in the backend's own
   * sentences — one another application already has, one two Actions both ask
   * for, one that is not a combination at all. Read back with the catalogue
   * every time, because claiming them is what asking for the catalogue does.
   */
  let unclaimedHotkeys = $state<string[]>([]);
  /**
   * The sentence a desktop that will not let Demysto read a Selection is owed,
   * `null` everywhere else (user story 56). ADR-0003 puts it here as well as in
   * the Palette: the Palette says it to somebody who has just pressed the
   * Hotkey, and this says it to somebody working out what the tool does.
   */
  let clipboardOnly = $state<string | null>(null);
  let editing = $state<Editing | null>(null);
  /**
   * Which Hotkey field is being recorded into, `null` for neither. While one is,
   * every keypress belongs to it rather than to the window: the combination
   * somebody wants is quite likely one that already means something here.
   */
  let recording = $state<"palette" | "action" | null>(null);
  /** The Hotkey that opens the Palette, empty for the one Demysto comes with. */
  let paletteHotkey = $state("");
  /**
   * How many characters a Selection may hold before Demysto says so.
   *
   * `null` is the file stating nothing, which leaves Demysto's own figure
   * deciding; zero is somebody who would rather not be told at all.
   */
  let largeSelection = $state<number | null>(null);
  /** Demysto's own figure, so that the field can say what leaving it empty means. */
  let largeSelectionDefault = $state(0);
  /**
   * The language the settings ask for, empty for following the operating
   * system.
   *
   * What the file states rather than what is being spoken: an environment
   * variable can be fixing the second, and a field that showed it would report
   * a choice nobody made — the same reason the key field shows where a key is
   * rather than the key.
   */
  let language = $state("");
  /**
   * The language the environment fixes, `null` where nothing is exported. The
   * field is still offered where it is set: what is written goes on being what
   * the file says, and is what will be spoken the moment the variable is not.
   */
  let languageFixed = $state<Exported | null>(null);
  /** Where the logs are, and what went wrong opening the folder. */
  let logsProblem = $state<string | null>(null);
  /** The Provider the window was opened at, so that it can be shown as such. */
  let wanted = $state<string | null>(null);
  /** The listener that carries that name, for as long as this window lives. */
  let listening: Promise<UnlistenFn> | null = null;
  /** What opens the Palette when nothing states otherwise, as it is read. */
  let paletteDefault = $state("");
  /** The keys a Hotkey may be on its own — the backend decides which. */
  let bareKeys = $state<ReadonlySet<string>>(new Set());
  /** What went wrong with the last Action saved, in the backend's own words. */
  let actionProblem = $state<string | null>(null);
  let actionSaving = $state(false);
  /**
   * The settings as the file holds them, which is where the Models an Action
   * can bind come from. Taken from what was saved rather than from the
   * Providers on screen: an Action binding a Model that has not been saved yet
   * is a binding the backend would refuse, and offering it would be inviting
   * that.
   */
  let savedSettings = $state<Settings | null>(null);

  /** What went wrong with the last save, in the words the backend chose. */
  let problem = $state<string | null>(null);
  let saving = $state(false);
  let saved = $state(false);
  /** Whether the settings have been read at all, so that an empty file and a
      window that has not loaded do not look the same. */
  let read = $state(false);
  /**
   * Why the settings file could not be read, when it could not be.
   *
   * Kept apart from a failed save because it stops this window doing anything
   * at all. Demysto will not write over a file it could not parse — that would
   * throw away whatever is in it, comments and keys alike — so the fields are
   * not offered, and the only honest instruction is to repair the file itself.
   */
  let unreadable = $state<string | null>(null);

  const FIELD =
    "w-full rounded border border-neutral-300 bg-transparent px-2 py-1 text-sm " +
    "outline-none focus:border-neutral-500 dark:border-neutral-700 " +
    "dark:focus:border-neutral-500";

  const BUTTON =
    "cursor-pointer rounded border border-neutral-300 px-2 py-1 text-xs " +
    "hover:bg-neutral-100 disabled:cursor-default disabled:opacity-40 " +
    "disabled:hover:bg-transparent dark:border-neutral-700 " +
    "dark:hover:bg-neutral-800 dark:disabled:hover:bg-transparent";

  onMount(async () => {
    presets = await offeredPresets();

    const reported = await status();
    where = reported.config_dir;
    largeSelectionDefault = reported.large_selection_default;
    clipboardOnly = said(reported.capturing);
    languageFixed = reported.language_env;

    // A refused key is reported in the Conversation and fixed here, so the
    // window is told which Provider it was opened for.
    listening = onProviderWanted((provider) => {
      wanted = provider;
      settle(provider);
    });

    const may = await allowed();
    paletteDefault = may.palette_default;
    bareKeys = new Set(may.no_modifier_needed);

    held(await catalogued());

    try {
      show(await configured());
    } catch (error) {
      unreadable = saidBy(error);
    }

    read = true;
  });

  // Taken down in its own hook rather than by returning one, because the mount
  // above waits on the backend and Svelte takes a cleanup only from one that
  // does not.
  onDestroy(() => listening?.then((off) => off()));

  /** Brings the Provider this window was opened for into view. */
  async function settle(provider: string) {
    await tick();

    document
      .querySelector(`[data-provider="${CSS.escape(provider)}"]`)
      ?.scrollIntoView({ block: "center" });
  }

  /** Opens the folder the logs are written in, so a bug report can carry them. */
  async function showLogs() {
    logsProblem = await sending(openLogs);
  }

  /** Takes the settings as the file holds them as the state of this window. */
  function show(settings: Settings) {
    savedSettings = settings;
    drafts = settings.providers.map(drafted);
    defaultModel = settings.default_model ?? "";
    defaultVisionModel = settings.default_vision_model ?? "";
    paletteHotkey = settings.palette_hotkey ?? "";
    largeSelection = settings.large_selection;
    language = settings.language ?? "";
  }

  /** Takes the catalogue as the directory holds it as the state of this window. */
  function held(catalogue: Catalogue) {
    actions = catalogue.actions;
    unreadableActions = catalogue.unreadable;
    unclaimedHotkeys = catalogue.unclaimed;
  }

  function drafted(provider: ConfiguredProvider): Draft {
    return {
      was: provider.name,
      name: provider.name,
      base_url: provider.base_url ?? "",
      preset: provider.preset ?? "",
      api_key_env: provider.api_key_env ?? "",
      models: provider.models.map((model) => ({ ...model })),
      standing: provider.key,
      typed: "",
      forgetting: false,
      offered: null,
      trying: provider.models[0]?.id ?? "",
      asking: false,
      said: null,
    };
  }

  /** What of a draft gets written. */
  function edited(draft: Draft): ProviderEdit {
    return {
      was: draft.was,
      name: draft.name,
      base_url: draft.base_url,
      preset: draft.preset,
      api_key_env: draft.api_key_env,
      api_key: key(draft),
      models: draft.models,
    };
  }

  /**
   * What a save does to this Provider's key. Typing one replaces whatever the
   * file holds; typing nothing leaves it alone, which is the ordinary case and
   * the reason the field can start empty at all.
   */
  function key(draft: Draft): KeyEdit {
    if (draft.typed.trim() !== "") return { action: "set", key: draft.typed };

    return draft.forgetting ? { action: "forget" } : { action: "keep" };
  }

  /**
   * Every Model configured here, by the name it is nominated with.
   *
   * The `<provider>/<model>` shape is composed here rather than asked for,
   * because the list has to include Models added since the last save and a
   * question per keystroke would be a poor trade for one separator. What it
   * composes is checked where it matters: a nomination naming no Model is
   * refused by the save, in `settings::nominating`.
   */
  const nominable = $derived(
    drafts.flatMap((draft) =>
      draft.models
        // A row added and not yet typed into is not a Model to nominate: the
        // save refuses one with no name, and offering "a provider/" here would
        // be inviting exactly that.
        .filter((model) => model.id.trim() !== "")
        .map((model) => ({
          name: `${draft.name}/${model.id}`,
          vision: model.vision,
        })),
    ),
  );

  /** What the key field says instead of the key. */
  function about(draft: Draft): string {
    if (draft.forgetting) return t("settings-key-going");

    switch (draft.standing.state) {
      case "in_file":
        return t("settings-key-in-file");
      case "in_environment":
        return t("settings-key-in-environment", {
          variable: draft.standing.variable,
        });
      case "not_needed":
        return t("settings-key-not-needed");
      case "missing":
        return t("settings-key-missing");
    }
  }

  function add() {
    drafts.push({
      was: null,
      name: "",
      base_url: "",
      preset: "",
      api_key_env: "",
      models: [],
      standing: { state: "missing" },
      typed: "",
      forgetting: false,
      offered: null,
      trying: "",
      asking: false,
      said: null,
    });
  }

  function remove(at: number) {
    drafts.splice(at, 1);
  }

  /** Fills in what a preset knows, so that picking one is the whole of setup. */
  function picked(draft: Draft) {
    draft.offered = null;
    draft.said = null;

    // Only into a name nobody has typed over: a Provider called something of
    // the user's own is not renamed by their changing its service.
    const preset = presets.find((preset) => preset.name === draft.preset);
    if (preset && draft.name.trim() === "") draft.name = preset.name;
  }

  /** Adds a Model to a Provider, unless it already offers one by that name. */
  function offer(draft: Draft, id: string) {
    if (draft.models.some((model) => model.id === id)) return;

    draft.models.push({ id, vision: false });
    if (draft.trying === "") draft.trying = id;
  }

  /** Takes a Model off a Provider. Named for the Model, because `forgetting`
      next door is about the key. */
  function stopOffering(draft: Draft, at: number) {
    const [gone] = draft.models.splice(at, 1);
    if (draft.trying === gone.id) draft.trying = draft.models[0]?.id ?? "";
  }

  /** Asks a Provider what it offers, as this window has it rather than as the
      file holds it: the commonest moment to want the list is before saving. */
  async function askForModels(draft: Draft) {
    draft.asking = true;
    draft.said = null;

    try {
      draft.offered = await providerModels(edited(draft));

      if (draft.offered.length === 0) {
        draft.said = { well: false, message: t("settings-provider-offers-nothing") };
      }
    } catch (error) {
      draft.said = { well: false, message: saidBy(error) };
    } finally {
      draft.asking = false;
    }
  }

  /** Puts the smallest real request to a Provider, to learn now rather than at
      the first Run whether the key works (user story 42). */
  async function verify(draft: Draft) {
    draft.asking = true;
    draft.said = null;

    try {
      await verifyProvider(edited(draft), draft.trying);
      draft.said = {
        well: true,
        message: t("settings-provider-answered", { model: draft.trying }),
      };
    } catch (error) {
      draft.said = { well: false, message: saidBy(error) };
    } finally {
      draft.asking = false;
    }
  }

  async function save() {
    saving = true;
    problem = null;

    try {
      // Shown from what came back rather than from what went out: a save is
      // finished when the file reads back, and what it reads back as is what
      // the next Run will use — a key that turned out to be in a variable
      // included.
      show(await saveSettings({
        providers: drafts.map(edited),
        default_model: defaultModel,
        default_vision_model: defaultVisionModel,
        palette_hotkey: paletteHotkey,
        // A blank field is the setting taken out of the file, which is not the
        // same as a zero somebody typed: one leaves Demysto's own figure
        // deciding, the other asks to be told nothing.
        large_selection: stated(largeSelection),
        language,
      }));

      // Asked for again because reading the catalogue is what claims the
      // Hotkeys: without this the Palette would answer to its old combination
      // until something else happened to read it. It also brings back the
      // sentences, which is how somebody learns that the Palette has just taken
      // a Hotkey an Action was using.
      held(await catalogued());

      saved = true;
      setTimeout(() => (saved = false), ACKNOWLEDGED);
    } catch (error) {
      problem = saidBy(error);
    } finally {
      saving = false;
    }
  }

  /**
   * A count as the file states it: what was typed, or nothing at all when the
   * field was left empty or filled with something that is not a count.
   *
   * A negative number is nothing rather than a refusal: the field is a number
   * of characters, and there is no reading of "-5 characters" worth writing
   * into somebody's settings or stopping their save over.
   */
  function stated(held: number | null): number | null {
    return held === null || !Number.isFinite(held) || held < 0
      ? null
      : Math.floor(held);
  }

  /**
   * Every Model configured, by the name an Action binds it with — from the file
   * rather than from the Providers on screen, for the reason `savedSettings`
   * exists.
   */
  const bindable = $derived(
    (savedSettings?.providers ?? []).flatMap((provider) =>
      provider.models.map((model) => `${provider.name}/${model.id}`),
    ),
  );

  /** What an Action being written starts as. */
  function write() {
    actionProblem = null;
    recording = null;
    editing = {
      standing: null,
      draft: {
        id: null,
        name: "",
        template: "",
        parameters: [],
        model: null,
        hotkey: null,
        accepts: ["text"],
      },
    };
  }

  /**
   * Opens an Action for editing.
   *
   * Everything it states is carried into the draft, the Hotkey and the
   * Selection kinds included — neither has a field here yet, and a save that
   * dropped what the file already said would be this window destroying what it
   * does not show.
   */
  function change(action: DefinedAction) {
    actionProblem = null;
    recording = null;
    editing = {
      standing: action.standing,
      draft: {
        id: action.id,
        name: action.name,
        template: action.template,
        parameters: action.parameters.map((parameter) => ({ ...parameter })),
        model: action.model,
        hotkey: action.hotkey,
        accepts: action.accepts,
      },
    };
  }

  /** Takes the Hotkey off this Action, which is the only way to have none. */
  function unbind() {
    if (editing) editing.draft.hotkey = null;
    recording = null;
  }

  /** Takes the Palette back to the Hotkey Demysto comes with. */
  function unbindPalette() {
    paletteHotkey = "";
    recording = null;
  }

  /** Leaves the Action being edited, whatever was being done to it. */
  function stopEditing() {
    editing = null;
    recording = null;
  }

  /** The language this window was drawn in, so that a change can be noticed. */
  let drawnIn = spokenTag();

  // A draft of a built-in holds what the built-in says, in the words it said
  // them in: its name, its Parameters' labels, and what they offer. Changing
  // the language leaves those the words of a language nobody chose — and saving
  // the draft afterwards would write them into an Override, leaving one Action
  // in the Palette speaking it for good. So the panel closes with the language
  // it was written in, and reopens in the new one.
  $effect(() => {
    if (spokenTag() === drawnIn) return;

    drawnIn = spokenTag();
    stopEditing();
  });

  function declare() {
    editing?.draft.parameters.push({ id: "", label: "", default: "" });
  }

  function stopDeclaring(at: number) {
    editing?.draft.parameters.splice(at, 1);
  }

  async function keep() {
    if (!editing) return;

    actionSaving = true;
    actionProblem = null;

    try {
      // Shown from what came back rather than from what went out, for the
      // reason a saved settings file is read back: a save is finished when the
      // directory reads back, and an Override that changed nothing leaves no
      // file at all.
      held(await saveAction(editing.draft));
      stopEditing();
    } catch (error) {
      actionProblem = saidBy(error);
    } finally {
      actionSaving = false;
    }
  }

  /** Deletes an Action of the user's own, or resets a built-in to how it was
      written by removing the Override over it. */
  async function forget(action: DefinedAction) {
    actionProblem = null;

    try {
      held(await deleteAction(action.id));
      if (editing?.draft.id === action.id) stopEditing();
    } catch (error) {
      actionProblem = saidBy(error);
    }
  }

  /** What the list calls an Action's standing, where it is worth calling
      anything: an Action nobody has touched needs no label. */
  function standing(action: DefinedAction): string | null {
    switch (action.standing) {
      case "built_in":
        return null;
      case "overridden":
        return t("settings-action-changed");
      case "authored":
        return t("settings-action-yours");
    }
  }

  function onKeydown(event: KeyboardEvent) {
    // While a Hotkey is being recorded every keypress is the Hotkey, including
    // the ones this window would otherwise act on: the combination the user
    // wants is quite likely one that already means something here.
    if (recording !== null) {
      event.preventDefault();
      record(event);
      return;
    }

    if (event.key !== "Escape") return;

    event.preventDefault();

    // Escape leaves what it is in: an Action being edited first, and the window
    // only once there is nothing left to back out of.
    if (editing) {
      stopEditing();
      return;
    }

    dismiss();
  }

  /**
   * Takes one keypress as the Hotkey being bound.
   *
   * A press that is not a combination yet — a modifier on its own, or a key
   * held without one — leaves the recording open: the user is mid-reach, and
   * stopping there would bind half of what they meant.
   */
  function record(event: KeyboardEvent) {
    const pressed = combination(event, bareKeys);

    // Escape on its own is the way out, and is why the recording does not need
    // a Cancel button of its own. Escape with a modifier is a combination like
    // any other.
    if (event.code === "Escape" && !pressed) {
      recording = null;
      return;
    }

    if (!pressed) return;

    if (recording === "palette") {
      paletteHotkey = pressed;
    } else if (editing) {
      editing.draft.hotkey = pressed;
    }

    recording = null;
  }
</script>

<svelte:window onkeydown={onKeydown} />

<main
  class="flex h-screen flex-col gap-4 bg-white p-6 font-sans text-neutral-900
         dark:bg-neutral-900 dark:text-neutral-100"
>
  <header class="flex items-baseline justify-between gap-3">
    <h1 class="text-sm font-semibold tracking-tight">{t("settings-title")}</h1>
    <span class="truncate text-xs opacity-40" title={where}>{where}</span>
  </header>

  <div class="flex min-h-0 flex-1 flex-col gap-6 overflow-y-auto">
    {#if unreadable}
      <!-- Demysto will not write over a file it could not parse: whatever is in
           it, comments and keys alike, would go. So nothing is offered here but
           what is wrong and where. -->
      <section class="flex flex-col gap-2">
        <p class="text-sm text-red-600 dark:text-red-400">{unreadable}</p>
        <p class="text-sm opacity-60">{t("settings-unreadable-file")}</p>
      </section>
    {:else}
    <section class="flex flex-col gap-3">
      <div class="flex items-baseline justify-between gap-3">
        <h2 class="text-xs font-semibold tracking-wide uppercase opacity-50">
          {t("settings-providers")}
        </h2>
        <button type="button" class={BUTTON} onclick={add}>
          {t("settings-add-provider")}
        </button>
      </div>

      {#each drafts as draft, at (at)}
        <!-- Named on the element, so that a window opened for one Provider —
             which is what a refused key does — can bring it into view and say
             which one it came here for. -->
        <article
          data-provider={draft.was ?? draft.name}
          class="flex flex-col gap-3 rounded-md border p-3
                 {(draft.was ?? draft.name) === wanted
            ? 'border-red-400 dark:border-red-500'
            : 'border-neutral-200 dark:border-neutral-700'}"
        >
          <div class="grid grid-cols-2 gap-3">
            <label class="flex flex-col gap-1">
              <span class="text-xs opacity-60">{t("settings-provider-name")}</span>
              <input
                bind:value={draft.name}
                class={FIELD}
                placeholder={t("settings-provider-name-example")}
              />
            </label>

            <label class="flex flex-col gap-1">
              <span class="text-xs opacity-60">{t("settings-provider-service")}</span>
              <select
                bind:value={draft.preset}
                onchange={() => picked(draft)}
                class={FIELD}
              >
                <option value="">{t("settings-provider-no-preset")}</option>
                {#each presets as preset (preset.name)}
                  <option value={preset.name}>
                    {preset.needs_key
                      ? preset.name
                      : t("settings-provider-preset-keyless", {
                          preset: preset.name,
                        })}
                  </option>
                {/each}
              </select>
            </label>

            <label class="col-span-2 flex flex-col gap-1">
              <span class="text-xs opacity-60">
                {draft.preset === ""
                  ? t("settings-provider-base-url")
                  : t("settings-provider-base-url-from-preset")}
              </span>
              <input
                bind:value={draft.base_url}
                class={FIELD}
                placeholder={presets.find((it) => it.name === draft.preset)
                  ?.base_url ?? t("settings-provider-base-url-example")}
              />
            </label>

            <label class="flex flex-col gap-1">
              <span class="text-xs opacity-60">{t("settings-provider-key")}</span>
              <input
                type="password"
                bind:value={draft.typed}
                oninput={() => (draft.forgetting = false)}
                class={FIELD}
                placeholder={about(draft)}
              />
            </label>

            <label class="flex flex-col gap-1">
              <span class="text-xs opacity-60">
                {t("settings-provider-key-variable")}
              </span>
              <input
                bind:value={draft.api_key_env}
                class={FIELD}
                placeholder={presets.find((it) => it.name === draft.preset)
                  ?.variable ?? t("settings-provider-key-variable-example")}
              />
            </label>
          </div>

          {#if draft.standing.state === "in_file"}
            <p class="text-xs opacity-50">
              <button
                type="button"
                class="cursor-pointer underline underline-offset-2"
                onclick={() => {
                  draft.forgetting = !draft.forgetting;
                  draft.typed = "";
                }}
              >
                {draft.forgetting
                  ? t("settings-keep-key")
                  : t("settings-remove-key")}
              </button>
            </p>
          {/if}

          <div class="flex flex-col gap-2">
            <div class="flex items-baseline justify-between gap-3">
              <span class="text-xs opacity-60">{t("settings-models")}</span>
              <div class="flex gap-2">
                <button
                  type="button"
                  class={BUTTON}
                  disabled={draft.asking}
                  onclick={() => askForModels(draft)}
                >
                  {t("settings-fetch-models")}
                </button>
                <button
                  type="button"
                  class={BUTTON}
                  disabled={draft.asking || draft.trying === ""}
                  onclick={() => verify(draft)}
                >
                  {t("settings-verify-key")}
                </button>
              </div>
            </div>

            <ul class="flex flex-col gap-1">
              {#each draft.models as model, index (index)}
                <li class="flex items-center gap-2">
                  <input bind:value={model.id} class="{FIELD} flex-1" />

                  <label class="flex items-center gap-1 text-xs opacity-70">
                    <input type="checkbox" bind:checked={model.vision} />
                    {t("settings-model-sees-images")}
                  </label>

                  <label class="flex items-center gap-1 text-xs opacity-70">
                    <input
                      type="radio"
                      name="verifying-{at}"
                      value={model.id}
                      bind:group={draft.trying}
                    />
                    {t("settings-model-verify-with")}
                  </label>

                  <button
                    type="button"
                    class={BUTTON}
                    onclick={() => stopOffering(draft, index)}
                  >
                    {t("settings-remove-model")}
                  </button>
                </li>
              {:else}
                <li class="text-xs opacity-50">{t("settings-no-models")}</li>
              {/each}
            </ul>

            <div class="flex gap-2">
              <button type="button" class={BUTTON} onclick={() => offer(draft, "")}>
                {t("settings-add-model")}
              </button>
              <button type="button" class={BUTTON} onclick={() => remove(at)}>
                {t("settings-remove-provider")}
              </button>
            </div>

            {#if draft.asking}
              <p class="text-xs opacity-50">{t("settings-asking-provider")}</p>
            {:else if draft.said}
              <p
                class="text-xs {draft.said.well
                  ? 'text-green-700 dark:text-green-400'
                  : 'text-red-600 dark:text-red-400'}"
              >
                {draft.said.message}
              </p>
            {/if}

            {#if draft.offered && draft.offered.length > 0}
              <ul class="flex max-h-32 flex-wrap gap-1 overflow-y-auto">
                {#each draft.offered as id (id)}
                  <li>
                    <button
                      type="button"
                      class="cursor-pointer rounded bg-neutral-100 px-2 py-0.5 text-xs
                             hover:bg-neutral-200 dark:bg-neutral-800
                             dark:hover:bg-neutral-700"
                      onclick={() => offer(draft, id)}
                    >
                      {id}
                    </button>
                  </li>
                {/each}
              </ul>
            {/if}
          </div>
        </article>
      {:else}
        <p class="text-sm opacity-50">
          {read ? t("settings-no-providers") : t("settings-reading")}
        </p>
      {/each}
    </section>

    <section class="flex flex-col gap-3">
      <h2 class="text-xs font-semibold tracking-wide uppercase opacity-50">
        {t("settings-defaults")}
      </h2>

      <div class="grid grid-cols-2 gap-3">
        <label class="flex flex-col gap-1">
          <span class="text-xs opacity-60">{t("settings-default-model")}</span>
          <select bind:value={defaultModel} class={FIELD}>
            <option value="">{t("settings-model-none")}</option>
            {#each nominable as model (model.name)}
              <option value={model.name}>{model.name}</option>
            {/each}
          </select>
        </label>

        <label class="flex flex-col gap-1">
          <span class="text-xs opacity-60">
            {t("settings-default-vision-model")}
          </span>
          <select bind:value={defaultVisionModel} class={FIELD}>
            <option value="">{t("settings-model-none")}</option>
            {#each nominable as model (model.name)}
              <option value={model.name}>
                {model.vision
                  ? model.name
                  : t("settings-model-does-not-see", { model: model.name })}
              </option>
            {/each}
          </select>
        </label>
      </div>

      <label class="flex flex-col gap-1">
        <span class="text-xs opacity-60">{t("settings-large-selection")}</span>
        <input
          type="number"
          min="0"
          bind:value={largeSelection}
          placeholder={t("settings-large-selection-default", {
            characters: largeSelectionDefault,
          })}
          class="{FIELD} max-w-64"
        />
        <span class="text-xs opacity-50">
          {t("settings-large-selection-detail")}
        </span>
      </label>
    </section>

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

      {#if clipboardOnly}
        <!-- Both halves of what Wayland costs, together and where the Hotkey is
             set, because both are answers to "why did pressing it do that?" —
             see ADR-0003. -->
        <p class="text-xs opacity-50">{clipboardOnly}</p>

        <p class="text-xs opacity-50">{t("settings-wayland-hotkeys")}</p>
      {/if}

      {#if !unreadable}
        <div class="flex flex-col gap-1">
          <span class="text-xs opacity-60">{t("settings-palette-hotkey")}</span>
          <div class="flex items-center gap-2">
            <span
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
              disabled={recording !== null}
              onclick={() => (recording = "palette")}
            >
              {t("settings-hotkey-record")}
            </button>

            <button
              type="button"
              class={BUTTON}
              disabled={!paletteHotkey}
              onclick={unbindPalette}
            >
              {t("settings-hotkey-clear")}
            </button>
          </div>
          <span class="text-xs opacity-50">
            {t("settings-hotkey-rule")}
            {t("settings-palette-hotkey-detail")}
          </span>
        </div>
      {/if}

      {#each unclaimedHotkeys as said (said)}
        <p class="text-xs text-red-600 dark:text-red-400">{said}</p>
      {/each}
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
      <div class="flex items-baseline justify-between gap-3">
        <h2 class="text-xs font-semibold tracking-wide uppercase opacity-50">
          {t("settings-actions")}
        </h2>
        <button type="button" class={BUTTON} onclick={write}>
          {t("settings-write-action")}
        </button>
      </div>

      <!-- Through `@html` for the `<code>` in it, which is markup a
           translation has to be able to put where its own sentence wants it.
           The catalogues are this repository's own files, not anything a user
           or a Model wrote: the two places untrusted text is rendered are the
           answer and the Selection, and neither comes through here. -->
      <p class="text-xs opacity-50">{@html t("settings-actions-detail")}</p>

      {#each unreadableActions as said (said)}
        <p class="text-xs text-red-600 dark:text-red-400">{said}</p>
      {/each}

      <ul class="flex flex-col gap-1">
        {#each actions as action (action.id)}
          <li
            class="flex items-center gap-2 rounded border border-neutral-200 px-2
                   py-1.5 dark:border-neutral-700"
          >
            <span class="flex-1 truncate text-sm" title={action.path ?? ""}>
              {action.name}
            </span>

            {#if standing(action)}
              <span class="text-xs opacity-40">{standing(action)}</span>
            {/if}

            {#if action.hotkey}
              <span class="truncate text-xs opacity-40">
                {reading(action.hotkey)}
              </span>
            {/if}

            {#if action.model}
              <span class="truncate text-xs opacity-40">{action.model}</span>
            {/if}

            <button
              type="button"
              class={BUTTON}
              onclick={() => change(action)}
              disabled={editing?.draft.id === action.id}
            >
              {t("settings-action-edit")}
            </button>

            {#if action.standing === "overridden"}
              <button type="button" class={BUTTON} onclick={() => forget(action)}>
                {t("settings-action-reset")}
              </button>
            {:else if action.standing === "authored"}
              <button type="button" class={BUTTON} onclick={() => forget(action)}>
                {t("settings-action-delete")}
              </button>
            {/if}
          </li>
        {/each}
      </ul>

      {#if editing}
        <article
          class="flex flex-col gap-3 rounded-md border border-neutral-300 p-3
                 dark:border-neutral-600"
        >
          <div class="grid grid-cols-2 gap-3">
            <label class="flex flex-col gap-1">
              <span class="text-xs opacity-60">{t("settings-action-name")}</span>
              <input
                bind:value={editing.draft.name}
                class={FIELD}
                placeholder={t("settings-action-name-example")}
              />
            </label>

            <label class="flex flex-col gap-1">
              <span class="text-xs opacity-60">{t("settings-action-model")}</span>
              <select bind:value={editing.draft.model} class={FIELD}>
                <option value={null}>{t("settings-action-model-default")}</option>
                {#each bindable as model (model)}
                  <option value={model}>{model}</option>
                {/each}
              </select>
            </label>
          </div>

          <div class="flex flex-col gap-1">
            <span class="text-xs opacity-60">{t("settings-action-hotkey")}</span>
            <div class="flex items-center gap-2">
              <span
                class="{FIELD} flex-1 truncate {editing.draft.hotkey ||
                recording === 'action'
                  ? ''
                  : 'opacity-40'}"
              >
                {#if recording === "action"}
                  {t("settings-hotkey-recording")}
                {:else if editing.draft.hotkey}
                  {reading(editing.draft.hotkey)}
                {:else}
                  {t("settings-hotkey-none")}
                {/if}
              </span>

              <button
                type="button"
                class={BUTTON}
                disabled={recording !== null}
                onclick={() => (recording = "action")}
              >
                {t("settings-hotkey-record")}
              </button>

              <button
                type="button"
                class={BUTTON}
                disabled={!editing.draft.hotkey}
                onclick={unbind}
              >
                {t("settings-hotkey-clear")}
              </button>
            </div>
            <span class="text-xs opacity-50">
              {t("settings-hotkey-rule")}
              {t("settings-action-hotkey-detail")}
            </span>
          </div>

          <label class="flex flex-col gap-1">
            <span class="text-xs opacity-60">{t("settings-action-prompt")}</span>
            <textarea
              bind:value={editing.draft.template}
              rows="8"
              class="{FIELD} resize-y font-mono text-xs"
              placeholder={t("settings-action-prompt-example")}
            ></textarea>
          </label>

          <!-- Through `@html` for the reason the Actions note above is. -->
          <p class="text-xs opacity-50">
            {@html t("settings-action-prompt-detail")}
          </p>

          <div class="flex flex-col gap-2">
            <div class="flex items-baseline justify-between gap-3">
              <span class="text-xs opacity-60">{t("settings-parameters")}</span>
              <button type="button" class={BUTTON} onclick={declare}>
                {t("settings-declare-parameter")}
              </button>
            </div>

            <ul class="flex flex-col gap-1">
              {#each editing.draft.parameters as parameter, at (at)}
                <li class="flex items-center gap-2">
                  <input
                    bind:value={parameter.id}
                    class="{FIELD} flex-1 font-mono text-xs"
                    placeholder={t("settings-parameter-id-example")}
                  />
                  <input
                    bind:value={parameter.label}
                    class="{FIELD} flex-1"
                    placeholder={t("settings-parameter-label-example")}
                  />
                  <input
                    bind:value={parameter.default}
                    class="{FIELD} flex-1"
                    placeholder={t("settings-parameter-default-example")}
                  />
                  <button
                    type="button"
                    class={BUTTON}
                    onclick={() => stopDeclaring(at)}
                  >
                    {t("settings-remove-parameter")}
                  </button>
                </li>
              {:else}
                <li class="text-xs opacity-50">{t("settings-no-parameters")}</li>
              {/each}
            </ul>
          </div>

          {#if actionProblem}
            <p class="text-xs text-red-600 dark:text-red-400">{actionProblem}</p>
          {/if}

          <div class="flex items-center gap-2">
            <button
              type="button"
              class={BUTTON}
              disabled={actionSaving}
              onclick={keep}
            >
              {actionSaving ? t("settings-saving") : t("settings-save-action")}
            </button>
            <button type="button" class={BUTTON} onclick={stopEditing}>
              {t("settings-cancel")}
            </button>
            {#if editing.standing === "overridden"}
              <span class="text-xs opacity-50">{t("settings-reset-by-saving")}</span>
            {/if}
          </div>
        </article>
      {/if}
    </section>
  </div>

  <footer class="flex items-center justify-between gap-3">
    <p class="min-h-4 flex-1 text-xs">
      {#if problem}
        <span class="text-red-600 dark:text-red-400">{problem}</span>
      {:else if saved}
        <span class="opacity-50">{t("settings-saved")}</span>
      {:else}
        <span class="opacity-40">{t("settings-keys")}</span>
      {/if}
    </p>

    {#if !unreadable}
      <button type="button" class={BUTTON} disabled={saving} onclick={save}>
        {saving ? t("settings-saving") : t("settings-save")}
      </button>
    {/if}
  </footer>
</main>

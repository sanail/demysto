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
    type Catalogue,
    type ConfiguredModel,
    type ConfiguredProvider,
    type DefinedAction,
    type KeyEdit,
    type KeyStanding,
    type Preset,
    type ProviderEdit,
    type Settings,
  } from "../lib/ipc";
  import { combination, reading } from "../lib/hotkey";
  import { saidBy, sending } from "../lib/sending";
  import type { UnlistenFn } from "@tauri-apps/api/event";

  /** How long the window says a save landed. */
  const ACKNOWLEDGED = 1600;

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

  /**
   * What both Hotkey fields say about what may be recorded. One sentence rather
   * than two, because it is one rule — and honest about which keys are worth
   * trying: an operating system answers to its own volume and media keys long
   * before Demysto sees them.
   */
  const HOTKEY_RULE =
    "Hold at least one modifier, or press a key that types nothing on its own — " +
    "F13 and above are the ones most keyboards can send.";

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
    if (draft.forgetting) return "Will be removed when you save";

    switch (draft.standing.state) {
      case "in_file":
        return "Held in the settings file — type to replace it";
      case "in_environment":
        return `Taken from ${draft.standing.variable}`;
      case "not_needed":
        return "This service has no keys";
      case "missing":
        return "No key yet";
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
        draft.said = { well: false, message: "It offers no Models." };
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
      draft.said = { well: true, message: `${draft.trying} answered.` };
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
        return "Changed";
      case "authored":
        return "Yours";
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
    <h1 class="text-sm font-semibold tracking-tight">Settings</h1>
    <span class="truncate text-xs opacity-40" title={where}>{where}</span>
  </header>

  <div class="flex min-h-0 flex-1 flex-col gap-6 overflow-y-auto">
    {#if unreadable}
      <!-- Demysto will not write over a file it could not parse: whatever is in
           it, comments and keys alike, would go. So nothing is offered here but
           what is wrong and where. -->
      <section class="flex flex-col gap-2">
        <p class="text-sm text-red-600 dark:text-red-400">{unreadable}</p>
        <p class="text-sm opacity-60">
          Settings will not write over a file it cannot read, so nothing here can
          be edited until that file is repaired. Open it, fix what it says, and
          reopen this window.
        </p>
      </section>
    {:else}
    <section class="flex flex-col gap-3">
      <div class="flex items-baseline justify-between gap-3">
        <h2 class="text-xs font-semibold tracking-wide uppercase opacity-50">
          Providers
        </h2>
        <button type="button" class={BUTTON} onclick={add}>Add a Provider</button>
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
              <span class="text-xs opacity-60">Name</span>
              <input bind:value={draft.name} class={FIELD} placeholder="openai" />
            </label>

            <label class="flex flex-col gap-1">
              <span class="text-xs opacity-60">Service</span>
              <select
                bind:value={draft.preset}
                onchange={() => picked(draft)}
                class={FIELD}
              >
                <option value="">No preset</option>
                {#each presets as preset (preset.name)}
                  <option value={preset.name}>
                    {preset.name}{preset.needs_key ? "" : " (no key)"}
                  </option>
                {/each}
              </select>
            </label>

            <label class="col-span-2 flex flex-col gap-1">
              <span class="text-xs opacity-60">
                Base URL{draft.preset === "" ? "" : " — leave empty to use the preset's"}
              </span>
              <input
                bind:value={draft.base_url}
                class={FIELD}
                placeholder={presets.find((it) => it.name === draft.preset)
                  ?.base_url ?? "https://api.example.com/v1"}
              />
            </label>

            <label class="flex flex-col gap-1">
              <span class="text-xs opacity-60">API key</span>
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
                Or the environment variable holding it
              </span>
              <input
                bind:value={draft.api_key_env}
                class={FIELD}
                placeholder={presets.find((it) => it.name === draft.preset)
                  ?.variable ?? "MY_API_KEY"}
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
                  ? "Keep the key in the file"
                  : "Remove the key from the file"}
              </button>
            </p>
          {/if}

          <div class="flex flex-col gap-2">
            <div class="flex items-baseline justify-between gap-3">
              <span class="text-xs opacity-60">Models</span>
              <div class="flex gap-2">
                <button
                  type="button"
                  class={BUTTON}
                  disabled={draft.asking}
                  onclick={() => askForModels(draft)}
                >
                  Fetch
                </button>
                <button
                  type="button"
                  class={BUTTON}
                  disabled={draft.asking || draft.trying === ""}
                  onclick={() => verify(draft)}
                >
                  Verify key
                </button>
              </div>
            </div>

            <ul class="flex flex-col gap-1">
              {#each draft.models as model, index (index)}
                <li class="flex items-center gap-2">
                  <input bind:value={model.id} class="{FIELD} flex-1" />

                  <label class="flex items-center gap-1 text-xs opacity-70">
                    <input type="checkbox" bind:checked={model.vision} />
                    Sees images
                  </label>

                  <label class="flex items-center gap-1 text-xs opacity-70">
                    <input
                      type="radio"
                      name="verifying-{at}"
                      value={model.id}
                      bind:group={draft.trying}
                    />
                    Verify with
                  </label>

                  <button
                    type="button"
                    class={BUTTON}
                    onclick={() => stopOffering(draft, index)}
                  >
                    Remove
                  </button>
                </li>
              {:else}
                <li class="text-xs opacity-50">
                  No Model yet. Fetch the list, or add one by hand.
                </li>
              {/each}
            </ul>

            <div class="flex gap-2">
              <button type="button" class={BUTTON} onclick={() => offer(draft, "")}>
                Add a Model
              </button>
              <button type="button" class={BUTTON} onclick={() => remove(at)}>
                Remove this Provider
              </button>
            </div>

            {#if draft.asking}
              <p class="text-xs opacity-50">Asking the Provider…</p>
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
          {read
            ? "No Provider is configured yet. Add one to start asking things."
            : "Reading the settings…"}
        </p>
      {/each}
    </section>

    <section class="flex flex-col gap-3">
      <h2 class="text-xs font-semibold tracking-wide uppercase opacity-50">
        Defaults
      </h2>

      <div class="grid grid-cols-2 gap-3">
        <label class="flex flex-col gap-1">
          <span class="text-xs opacity-60">
            Default Model — what an Action with no Model of its own uses
          </span>
          <select bind:value={defaultModel} class={FIELD}>
            <option value="">None</option>
            {#each nominable as model (model.name)}
              <option value={model.name}>{model.name}</option>
            {/each}
          </select>
        </label>

        <label class="flex flex-col gap-1">
          <span class="text-xs opacity-60">
            Default Vision Model — what an image uses instead
          </span>
          <select bind:value={defaultVisionModel} class={FIELD}>
            <option value="">None</option>
            {#each nominable as model (model.name)}
              <option value={model.name}>
                {model.name}{model.vision ? "" : " (does not see)"}
              </option>
            {/each}
          </select>
        </label>
      </div>

      <label class="flex flex-col gap-1">
        <span class="text-xs opacity-60">
          Warn above — how many characters a Selection may hold before Demysto
          says so
        </span>
        <input
          type="number"
          min="0"
          bind:value={largeSelection}
          placeholder={`${largeSelectionDefault} — what Demysto comes with`}
          class="{FIELD} max-w-64"
        />
        <span class="text-xs opacity-50">
          Nothing is ever cut and nothing is ever refused: the warning is there
          so that an accidental select-all is not silently paid for. Leave the
          field empty for Demysto's own figure, or set it to 0 to be told
          nothing.
        </span>
      </label>
    </section>
    {/if}

    <section class="flex flex-col gap-3">
      <h2 class="text-xs font-semibold tracking-wide uppercase opacity-50">
        Hotkeys
      </h2>

      {#if !unreadable}
        <div class="flex flex-col gap-1">
          <span class="text-xs opacity-60">
            The Palette — what opens it over whatever you are reading
          </span>
          <div class="flex items-center gap-2">
            <span
              class="{FIELD} flex-1 truncate {paletteHotkey ||
              recording === 'palette'
                ? ''
                : 'opacity-40'}"
            >
              {#if recording === "palette"}
                Press a combination… Esc to stop
              {:else if paletteHotkey}
                {reading(paletteHotkey)}
              {:else}
                {reading(paletteDefault)} — what Demysto comes with
              {/if}
            </span>

            <button
              type="button"
              class={BUTTON}
              disabled={recording !== null}
              onclick={() => (recording = "palette")}
            >
              Record
            </button>

            <button
              type="button"
              class={BUTTON}
              disabled={!paletteHotkey}
              onclick={unbindPalette}
            >
              Clear
            </button>
          </div>
          <span class="text-xs opacity-50">
            {HOTKEY_RULE} Saved by the Save button below, and answered to as soon
            as it is.
          </span>
        </div>
      {/if}

      {#each unclaimedHotkeys as said (said)}
        <p class="text-xs text-red-600 dark:text-red-400">{said}</p>
      {/each}
    </section>

    <section class="flex flex-col gap-3">
      <h2 class="text-xs font-semibold tracking-wide uppercase opacity-50">
        Logs
      </h2>

      <p class="text-xs opacity-50">
        Demysto keeps a local log of what it did — which Action, which Model,
        what went wrong — and never of what you were looking at or what a Model
        said. Nothing is sent anywhere. Attach these to a bug report.
      </p>

      <div>
        <button type="button" class={BUTTON} onclick={showLogs}>
          Open the log folder
        </button>
      </div>

      {#if logsProblem}
        <p class="text-xs text-red-600 dark:text-red-400">{logsProblem}</p>
      {/if}
    </section>

    <section class="flex flex-col gap-3">
      <div class="flex items-baseline justify-between gap-3">
        <h2 class="text-xs font-semibold tracking-wide uppercase opacity-50">
          Actions
        </h2>
        <button type="button" class={BUTTON} onclick={write}>
          Write an Action
        </button>
      </div>

      <p class="text-xs opacity-50">
        Each Action is a file of its own in <code>actions</code>, so one can be
        backed up or sent to somebody. Built-in Actions are not written there:
        changing one keeps only what you changed, and resetting it deletes that.
        An Action is saved on its own, not by the Save button below.
      </p>

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
              Edit
            </button>

            {#if action.standing === "overridden"}
              <button type="button" class={BUTTON} onclick={() => forget(action)}>
                Reset
              </button>
            {:else if action.standing === "authored"}
              <button type="button" class={BUTTON} onclick={() => forget(action)}>
                Delete
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
              <span class="text-xs opacity-60">Name — what the Palette lists</span>
              <input
                bind:value={editing.draft.name}
                class={FIELD}
                placeholder="Rewrite plainly"
              />
            </label>

            <label class="flex flex-col gap-1">
              <span class="text-xs opacity-60">
                Model — leave at the default unless this Action needs its own
              </span>
              <select bind:value={editing.draft.model} class={FIELD}>
                <option value={null}>Whatever the defaults say</option>
                {#each bindable as model (model)}
                  <option value={model}>{model}</option>
                {/each}
              </select>
            </label>
          </div>

          <div class="flex flex-col gap-1">
            <span class="text-xs opacity-60">
              Hotkey — runs this Action on what you have selected, with no
              Palette in the way
            </span>
            <div class="flex items-center gap-2">
              <span
                class="{FIELD} flex-1 truncate {editing.draft.hotkey ||
                recording === 'action'
                  ? ''
                  : 'opacity-40'}"
              >
                {#if recording === "action"}
                  Press a combination… Esc to stop
                {:else if editing.draft.hotkey}
                  {reading(editing.draft.hotkey)}
                {:else}
                  None — this Action is reached through the Palette
                {/if}
              </span>

              <button
                type="button"
                class={BUTTON}
                disabled={recording !== null}
                onclick={() => (recording = "action")}
              >
                Record
              </button>

              <button
                type="button"
                class={BUTTON}
                disabled={!editing.draft.hotkey}
                onclick={unbind}
              >
                Clear
              </button>
            </div>
            <span class="text-xs opacity-50">
              {HOTKEY_RULE} Parameters are not asked for on this path — each takes
              what it offers.
            </span>
          </div>

          <label class="flex flex-col gap-1">
            <span class="text-xs opacity-60">Prompt</span>
            <textarea
              bind:value={editing.draft.template}
              rows="8"
              class="{FIELD} resize-y font-mono text-xs"
              placeholder="Explain the text below. The text is in
{'{{'}selection_language{'}}'}; answer in {'{{'}ui_language{'}}'}.

{'{{'}selection{'}}'}"
            ></textarea>
          </label>

          <p class="text-xs opacity-50">
            <code>{"{{selection}}"}</code> is what you selected;
            <code>{"{{ui_language}}"}</code> and
            <code>{"{{selection_language}}"}</code> are the language you read and
            the one it turned out to be in. Anything else in double braces is a
            Parameter, which the Palette asks for before the Run — declare it
            below.
          </p>

          <div class="flex flex-col gap-2">
            <div class="flex items-baseline justify-between gap-3">
              <span class="text-xs opacity-60">Parameters</span>
              <button type="button" class={BUTTON} onclick={declare}>
                Declare a Parameter
              </button>
            </div>

            <ul class="flex flex-col gap-1">
              {#each editing.draft.parameters as parameter, at (at)}
                <li class="flex items-center gap-2">
                  <input
                    bind:value={parameter.id}
                    class="{FIELD} flex-1 font-mono text-xs"
                    placeholder="target"
                  />
                  <input
                    bind:value={parameter.label}
                    class="{FIELD} flex-1"
                    placeholder="Into which language?"
                  />
                  <input
                    bind:value={parameter.default}
                    class="{FIELD} flex-1"
                    placeholder="What it offers"
                  />
                  <button
                    type="button"
                    class={BUTTON}
                    onclick={() => stopDeclaring(at)}
                  >
                    Remove
                  </button>
                </li>
              {:else}
                <li class="text-xs opacity-50">
                  None. This Action runs the moment it is chosen.
                </li>
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
              {actionSaving ? "Saving\u2026" : "Save this Action"}
            </button>
            <button type="button" class={BUTTON} onclick={stopEditing}>
              Cancel
            </button>
            {#if editing.standing === "overridden"}
              <span class="text-xs opacity-50">
                Saving this with nothing changed puts the built-in back.
              </span>
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
        <span class="opacity-50">Saved.</span>
      {:else}
        <span class="opacity-40">Esc to close</span>
      {/if}
    </p>

    {#if !unreadable}
      <button type="button" class={BUTTON} disabled={saving} onclick={save}>
        {saving ? "Saving…" : "Save"}
      </button>
    {/if}
  </footer>
</main>

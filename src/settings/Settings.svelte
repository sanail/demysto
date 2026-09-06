<script lang="ts">
  import { onDestroy, onMount, tick } from "svelte";
  import {
    autostart,
    catalogue as catalogued,
    dismiss,
    hotkeys as allowed,
    onProviderWanted,
    onSettingsSaved,
    onUpdateOffered,
    presets as offeredPresets,
    saveSettings,
    setAutostart,
    settings as configured,
    status,
    updateOffered,
    type Capturing,
    type Catalogue,
    type DefinedAction,
    type Exported,
    type Preset,
    type Settings,
  } from "../lib/ipc";
  import { combination } from "../lib/hotkey";
  import { t } from "../lib/i18n.svelte";
  import { saidBy, sending } from "../lib/sending";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import About from "./About.svelte";
  import Actions from "./Actions.svelte";
  import General from "./General.svelte";
  import Models from "./Models.svelte";
  import { drafted, edited, type Draft, type Editing } from "./drafts";
  import { BUTTON } from "./style";

  /** How long the window says a save landed. */
  const ACKNOWLEDGED = 1600;

  /** What a Capture on this desktop cannot do, where there is such a thing. */
  function said(capturing: Capturing): string | null {
    return capturing.reads === "clipboard_only"
      ? t("capture-clipboard-only")
      : null;
  }

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
  /**
   * Whether Demysto is in the login items, as the system last answered, and
   * what it said the last time it would not change them.
   */
  let autostartWanted = $state(false);
  let autostartProblem = $state<string | null>(null);
  /**
   * How many times the box has been acted on, and what the login items were
   * last asked — the two things that keep a reading and a change from
   * answering out of turn (ticket 27).
   *
   * Both are needed and neither is enough. One click on a window that did not
   * have focus sends both questions at once: `focus` on the way down asks what
   * the list says, `change` on the way up changes it. That reading was taken
   * before the change was made, so it must not be allowed to land after it —
   * which is what counting the clicks is for. And two clicks in quick
   * succession send two changes with no ordering between them, where the
   * system has to be left holding the one made last — which is what asking one
   * thing at a time is for.
   */
  let acted = 0;
  let settling: Promise<unknown> = Promise.resolve();
  /** The version this is, which is the half of an update question nobody else answers. */
  let version = $state("");
  /** The newer version there is, `null` where there is none to be had. */
  let newer = $state<string | null>(null);
  /** The Provider the window was opened at, so that it can be shown as such. */
  let wanted = $state<string | null>(null);
  /** The listener that carries that name, for as long as this window lives. */
  let listening: Promise<UnlistenFn> | null = null;
  /** The same, for what the backend's own update checks find. */
  let watching: Promise<UnlistenFn> | null = null;
  /** The same, for the settings as each save leaves them. */
  let following: Promise<UnlistenFn> | null = null;
  /** What opens the Palette when nothing states otherwise, as it is read. */
  let paletteDefault = $state("");
  /** The keys a Hotkey may be on its own — the backend decides which. */
  let bareKeys = $state<ReadonlySet<string>>(new Set());
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

  onMount(async () => {
    presets = await offeredPresets();

    const reported = await status();
    version = reported.version;
    where = reported.config_dir;
    largeSelectionDefault = reported.large_selection_default;
    clipboardOnly = said(reported.capturing);
    languageFixed = reported.language_env;
    // The first thing this window hears about the login items, and the only
    // one with nothing to fall back on: a system that will not say is drawn as
    // the offer to turn autostart on, which is what it was drawn as when the
    // backend answered no on its behalf (ticket 27).
    autostartWanted = (await inTheList()) ?? false;

    // A refused key is reported in the Conversation and fixed here, so the
    // window is told which Provider it was opened for.
    listening = onProviderWanted((provider) => {
      wanted = provider;
      settle(provider);
    });

    // The check on the way up answers after this window has loaded, so what it
    // finds arrives as an event rather than being waited for here.
    //
    // Listened for before what it found is read, and the registration waited
    // on: an answer landing between the two would otherwise be emitted to
    // nobody, and the window would go on showing that there is no update until
    // Demysto is restarted — closing this window hides it rather than
    // unloading it. Which of the two arrives first costs nothing, because the
    // version is held before it is announced.
    watching = onUpdateOffered((version) => (newer = version));
    await watching;

    // Every save, whoever made it — which is how the Provider the first-run
    // flow configures reaches a window that loaded its page before there was
    // one. Registered before the file is read below and waited on for the
    // reason above: a save landing between the two would be emitted to nobody.
    //
    // A settings file that arrives is a settings file that was read, so a
    // window that had nothing to offer because it could not be read has
    // something to offer now.
    following = onSettingsSaved((settings) => {
      unreadable = null;
      show(settings);
    });
    await following;

    newer = await updateOffered();

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
  onDestroy(() => {
    listening?.then((off) => off());
    watching?.then((off) => off());
    following?.then((off) => off());
  });

  /** Brings the Provider this window was opened for into view. */
  async function settle(provider: string) {
    await tick();

    document
      .querySelector(`[data-provider="${CSS.escape(provider)}"]`)
      ?.scrollIntoView({ block: "center" });
  }

  /**
   * Puts Demysto into the login items, or takes it out, as the box is ticked.
   *
   * Not held for the Save button, alone among the fields above it: the login
   * items are the operating system's list and not a line in the settings file,
   * so a save would have nothing of this to write, and a box waiting for one
   * would be a choice with nowhere to land. The sentence below it says so.
   */
  async function autostartIs(wanted: boolean) {
    const mine = ++acted;
    const was = autostartWanted;
    autostartWanted = wanted;

    const refused = await inTurn(() => sending(() => setAutostart(wanted)));

    // A click that has been overtaken says nothing about the list any more:
    // the click after it is the answer, and it is still on its way.
    if (mine !== acted) return;

    autostartProblem = refused;
    if (refused === null) return;

    // What the system says it did, rather than what it was asked for: a
    // refusal leaves the box where it was rather than lying about it. Where
    // the system will not say either, where it was is the best answer there
    // is — and it is the one the user had before they clicked.
    const said = await inTheList();
    if (mine === acted) autostartWanted = said ?? was;
  }

  /**
   * Asks the login items again, because they are edited somewhere else too.
   *
   * This window's page is loaded once at startup and hidden rather than closed,
   * so the reading taken at mount would otherwise stand for the whole session —
   * and this is the one thing here that the operating system's own settings
   * change as readily as Demysto does. Focus is what catches both ways back:
   * the window being shown, and somebody returning from that pane to a window
   * that never went away.
   */
  async function readAutostart() {
    const mine = acted;
    const said = await inTheList();

    // A reading the box has been clicked over says what the list held before
    // the click, and a list that would not say says nothing at all. Neither is
    // worth putting on screen over what is there.
    if (mine !== acted || said === null) return;

    autostartWanted = said;

    // The sentence under the box was about the last change; this is a fresh
    // reading of what came of everything, so there is nothing left for it to
    // be about.
    autostartProblem = null;
  }

  /** What the login items say, and `null` where nothing could be got out of them. */
  function inTheList(): Promise<boolean | null> {
    return inTurn(autostart).catch(() => null);
  }

  /**
   * Asks the login items one thing at a time, in the order they were asked.
   *
   * The caller waits on the answer; what the queue is left holding is that
   * answer with its failure taken off, so that one question the system refused
   * does not stop every question after it from being asked.
   */
  function inTurn<T>(ask: () => Promise<T>): Promise<T> {
    const answer = settling.then(ask);

    settling = answer.catch(() => {});

    return answer;
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
  const bindableModels = $derived(
    (savedSettings?.providers ?? []).flatMap((provider) =>
      provider.models.map((model) => `${provider.name}/${model.id}`),
    ),
  );

  /** Leaves the Action being edited, whatever was being done to it. */
  function stopEditing() {
    editing = null;
    recording = null;
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

    // Escape on its own is the way out — the keyboard's half of the way out
    // the Record button offers while it is recording (user story 66). Escape
    // with a modifier is a combination like any other.
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

<svelte:window onkeydown={onKeydown} onfocus={readAutostart} />

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
      <Models
        bind:drafts
        bind:defaultModel
        bind:defaultVisionModel
        bind:largeSelection
        {presets}
        {wanted}
        {read}
        {largeSelectionDefault}
      />
    {/if}

    <General
      bind:language
      bind:paletteHotkey
      bind:recording
      {unreadable}
      {languageFixed}
      {clipboardOnly}
      {paletteDefault}
      {unclaimedHotkeys}
      {autostartWanted}
      {autostartProblem}
      onAutostart={autostartIs}
    />

    <About {version} bind:newer />

    <Actions
      bind:editing
      bind:recording
      {actions}
      {unreadableActions}
      {bindableModels}
      {held}
      {stopEditing}
    />
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

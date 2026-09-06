<script lang="ts">
  import {
    deleteAction,
    saveAction,
    type Catalogue,
    type DefinedAction,
  } from "../lib/ipc";
  import { reading } from "../lib/hotkey";
  import { spokenTag, t } from "../lib/i18n.svelte";
  import { saidBy } from "../lib/sending";
  import type { Editing } from "./drafts";
  import { BUTTON, FIELD } from "./style";

  let {
    actions,
    unreadableActions,
    bindableModels,
    editing = $bindable(),
    recording = $bindable(),
    held,
    stopEditing,
  }: {
    /** The Actions as the directory holds them. */
    actions: DefinedAction[];
    /** And what of it could not be read. */
    unreadableActions: string[];
    /** Every Model configured, by the name an Action binds it with. */
    bindableModels: string[];
    editing: Editing | null;
    /** Which Hotkey field is being recorded into, `null` for neither. */
    recording: "palette" | "action" | null;
    /** Takes a catalogue that has just been read as the state of the window. */
    held: (catalogue: Catalogue) => void;
    /** Leaves the Action being edited, whatever was being done to it. */
    stopEditing: () => void;
  } = $props();

  /** What went wrong with the last Action saved, in the backend's own words. */
  let actionProblem = $state<string | null>(null);
  let actionSaving = $state(false);

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
</script>

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
            autocorrect="off"
            placeholder={t("settings-action-name-example")}
          />
        </label>

        <label class="flex flex-col gap-1">
          <span class="text-xs opacity-60">{t("settings-action-model")}</span>
          <select bind:value={editing.draft.model} class={FIELD}>
            <option value={null}>{t("settings-action-model-default")}</option>
            {#each bindableModels as model (model)}
              <option value={model}>{model}</option>
            {/each}
          </select>
        </label>
      </div>

      <!-- The Palette's Hotkey row, again and identically: the caption
           names the value, the rule is on the button that asks for a
           combination, and both buttons say which row they are in — which
           matters most here, where a second Record and a second Clear are
           on screen at the same time as the Palette's. See the comment
           there for why any of it is needed. -->
      <div class="flex flex-col gap-1">
        <span id="action-hotkey" class="text-xs opacity-60">
          {t("settings-action-hotkey")}
        </span>
        <div class="flex items-center gap-2">
          <span
            role="status"
            aria-labelledby="action-hotkey"
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
            aria-describedby="action-hotkey action-hotkey-rule"
            disabled={recording !== null && recording !== "action"}
            onclick={() =>
              (recording = recording === "action" ? null : "action")}
          >
            {recording === "action"
              ? t("settings-hotkey-cancel")
              : t("settings-hotkey-record")}
          </button>

          <button
            type="button"
            class={BUTTON}
            aria-describedby="action-hotkey"
            disabled={!editing.draft.hotkey}
            onclick={unbind}
          >
            {t("settings-hotkey-clear")}
          </button>
        </div>
        <p id="action-hotkey-rule" class="text-xs opacity-50">
          {t("settings-hotkey-rule")}
          {t("settings-action-hotkey-detail")}
        </p>
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
                autocorrect="off"
                placeholder={t("settings-parameter-id-example")}
              />
              <input
                bind:value={parameter.label}
                class="{FIELD} flex-1"
                autocorrect="off"
                placeholder={t("settings-parameter-label-example")}
              />
              <input
                bind:value={parameter.default}
                class="{FIELD} flex-1"
                autocorrect="off"
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

<script lang="ts">
  import { tick, untrack } from "svelte";
  import { providerModels, verifyProvider, type Preset } from "../lib/ipc";
  import { t } from "../lib/i18n.svelte";
  import { saidBy } from "../lib/sending";
  import { edited, fresh, type Asked, type Draft } from "./drafts";
  import { BUTTON, FIELD } from "./style";

  let {
    drafts = $bindable(),
    presets,
    wanted,
    read,
    defaultModel = $bindable(),
    defaultVisionModel = $bindable(),
    largeSelection = $bindable(),
    largeSelectionDefault,
  }: {
    drafts: Draft[];
    presets: Preset[];
    /** The Provider the window was opened at, so that it can be shown as such. */
    wanted: string | null;
    /** Whether the settings have been read at all, so that an empty file and a
        window that has not loaded do not look the same. */
    read: boolean;
    defaultModel: string;
    defaultVisionModel: string;
    largeSelection: number | null;
    /** Demysto's own figure, so that the field can say what leaving it empty means. */
    largeSelectionDefault: number;
  } = $props();

  /**
   * Which Provider is open below the list, `null` for none.
   *
   * By position, like the list itself: a Provider has no name of its own that
   * cannot be typed over, and the one thing the file knows it by is what it
   * was called when it was read.
   */
  let editingAt = $state<number | null>(null);

  /** The Provider this panel was last brought here for, so that being brought
      here twice is told from a name that merely stayed the same. */
  let brought = $state<string | null>(null);

  // A Run refused for want of a key sends somebody here to fix one Provider.
  // Opening it is the whole of what they came for; scrolling to a row and
  // leaving it shut would be an instruction to go on clicking.
  $effect(() => {
    if (wanted === null || wanted === brought) return;

    brought = wanted;

    // Untracked because this is about the name that arrived, not about the
    // drafts: read plainly, every keystroke in a name field would re-run this
    // and shut whatever else had been opened since.
    const at = untrack(() =>
      drafts.findIndex((draft) => (draft.was ?? draft.name) === wanted),
    );

    if (at < 0) return;

    open(at);
  });

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

  /** Opens one Provider, and puts the keyboard where somebody would start. */
  function open(at: number) {
    editingAt = at;

    tick().then(() =>
      document.getElementById("provider-name")?.focus({ preventScroll: true }),
    );
  }

  function add() {
    drafts.push(fresh());
    open(drafts.length - 1);
  }

  function remove(at: number) {
    drafts.splice(at, 1);

    // The list is held by position, so taking one out moves everything after
    // it. A selection left where it was would be pointing at its neighbour.
    if (editingAt === at) editingAt = null;
    else if (editingAt !== null && editingAt > at) editingAt -= 1;
  }

  /** Fills in what a preset knows, so that picking one is the whole of setup. */
  function picked(draft: Draft) {
    draft.offered = null;
    draft.said = { models: null, key: null };

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
    draft.asking = "models";
    draft.said.models = null;

    try {
      draft.offered = await providerModels(edited(draft));

      if (draft.offered.length === 0) {
        draft.said.models = {
          well: false,
          message: t("settings-provider-offers-nothing"),
        };
      }
    } catch (error) {
      draft.said.models = { well: false, message: saidBy(error) };
    } finally {
      draft.asking = null;
    }
  }

  /** Puts the smallest real request to a Provider, to learn now rather than at
      the first Run whether the key works (user story 42). */
  async function verify(draft: Draft) {
    draft.asking = "key";
    draft.said.key = null;

    try {
      await verifyProvider(edited(draft), draft.trying);
      draft.said.key = {
        well: true,
        message: t("settings-provider-answered", { model: draft.trying }),
      };
    } catch (error) {
      draft.said.key = { well: false, message: saidBy(error) };
    } finally {
      draft.asking = null;
    }
  }
</script>

{#snippet answer(draft: Draft, asked: Asked)}
  {@const said = draft.said[asked]}
  {#if draft.asking === asked}
    <p class="text-xs opacity-50">{t("settings-asking-provider")}</p>
  {:else if said}
    <p
      class="text-xs {said.well
        ? 'text-green-700 dark:text-green-400'
        : 'text-red-600 dark:text-red-400'}"
    >
      {said.message}
    </p>
  {/if}
{/snippet}

<section class="flex flex-col gap-3">
  <div class="flex items-baseline justify-between gap-3">
    <h2 class="text-xs font-semibold tracking-wide uppercase opacity-50">
      {t("settings-providers")}
    </h2>
    <button type="button" class={BUTTON} onclick={add}>
      {t("settings-add-provider")}
    </button>
  </div>

  <ul class="flex flex-col gap-1">
    {#each drafts as draft, at (at)}
      <!-- Named on the row, so that a window opened for one Provider — which
           is what a refused key does — can bring it into view and say which
           one it came here for. -->
      <li
        data-provider={draft.was ?? draft.name}
        class="flex items-center gap-2 rounded border px-2 py-1.5
               {(draft.was ?? draft.name) === wanted
          ? 'border-red-400 dark:border-red-500'
          : 'border-neutral-200 dark:border-neutral-700'}"
      >
        <span class="flex-1 truncate text-sm">
          {draft.name || t("settings-provider-unnamed")}
        </span>

        {#if draft.preset}
          <span class="truncate text-xs opacity-40">{draft.preset}</span>
        {/if}

        {#if draft.models.length > 0}
          <span class="max-w-48 truncate text-xs opacity-40">
            {draft.models.map((model) => model.id).join(", ")}
          </span>
        {/if}

        <button
          type="button"
          class={BUTTON}
          onclick={() => open(at)}
          disabled={editingAt === at}
        >
          {t("settings-provider-edit")}
        </button>

        <button type="button" class={BUTTON} onclick={() => remove(at)}>
          {t("settings-remove-provider")}
        </button>
      </li>
    {:else}
      <li class="text-sm opacity-50">
        {read ? t("settings-no-providers") : t("settings-reading")}
      </li>
    {/each}
  </ul>

  <!--
    One Provider open at a time, below the list, the way an Action is opened
    below its own. Every Provider expanded at once is what this window used
    to be: two of them and the panel was longer than the window, with the
    fields of the one being edited somewhere in the middle of it.

    Every field this window writes an identifier from says
    `autocorrect="off"`, for the reason the Palette's fields do (ticket 21)
    and with a consequence of its own: macOS corrects what is typed into a
    WebKit field, and what it corrects is what gets written. Watched on a
    live desktop through the first-run flow, whose fields are these ones:
    "mock" was written down as "Mock" and "mock-small" as "Mock-small" —
    a Provider refusing a Model the user typed correctly.

    Three fields are left out, and each for a reason of its own: the key is
    a password field, which macOS corrects nothing in; the warning
    threshold takes a number; and an Action's prompt is the one thing here
    that IS prose, written in whole sentences for a Model to read.
  -->
  {#if editingAt !== null && drafts[editingAt]}
    {@const at = editingAt}
    {@const draft = drafts[at]}
    <article
      class="flex flex-col gap-3 rounded-md border border-neutral-300 p-3
             dark:border-neutral-600"
    >
      <div class="grid grid-cols-2 gap-3">
        <label class="flex flex-col gap-1">
          <span class="text-xs opacity-60">{t("settings-provider-name")}</span>
          <input
            id="provider-name"
            bind:value={draft.name}
            class={FIELD}
            autocorrect="off"
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
            autocorrect="off"
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
          />
          <!-- The state of the key is a line of its own and not the field's
               placeholder: the longest of the five does not fit a field this
               wide, and a placeholder goes the moment somebody types — which
               is the moment "held in the settings file" is worth reading. -->
          <span class="text-xs opacity-50">{about(draft)}</span>
        </label>

        <label class="flex flex-col gap-1">
          <span class="text-xs opacity-60">
            {t("settings-provider-key-variable")}
          </span>
          <input
            bind:value={draft.api_key_env}
            class={FIELD}
            autocorrect="off"
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
              disabled={draft.asking !== null}
              onclick={() => askForModels(draft)}
            >
              {t("settings-fetch-models")}
            </button>
          </div>
        </div>

        <ul class="flex flex-col gap-1">
          {#each draft.models as model, index (index)}
            <li class="flex items-center gap-2">
              <input
                bind:value={model.id}
                class="{FIELD} flex-1"
                autocorrect="off"
              />

              <label class="flex items-center gap-1 text-xs opacity-70">
                <input type="checkbox" bind:checked={model.vision} />
                {t("settings-model-sees-images")}
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

        {@render answer(draft, "models")}

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

        <!-- Removing a Provider is on its row and not here, where it used to
             sit beside "Add a Model": the row is where a Provider is picked
             out of the others, and it is the one place a Provider that is not
             open can be got rid of. -->
        <div class="flex gap-2">
          <button type="button" class={BUTTON} onclick={() => offer(draft, "")}>
            {t("settings-add-model")}
          </button>
        </div>

        <!-- Verifying the key is not one more thing to do to the list above, and
             a row it shared with "Add a Model" read as though it were. It is
             about the key: a key is proved by a real request (ADR-0008), which
             is why it needs a Model named at all, and the Model it names is the
             only reason it stands down here rather than beside the key field.
             The rule above it is what says the two are different things.

             The Model stands beside the button rather than as a radio on every
             Model's row, which was the one place that never said what it was
             for. It is also the shape the first-run window already asks in. -->
        <div
          class="flex flex-wrap items-center gap-2 border-t border-neutral-200
                 pt-3 dark:border-neutral-700"
        >
          <select
            bind:value={draft.trying}
            aria-label={t("settings-verify-which-model")}
            class="{FIELD} max-w-48 truncate"
          >
            <option value="">{t("settings-verify-which-model")}</option>
            {#each draft.models.filter((model) => model.id !== "") as model, index (index)}
              <option value={model.id}>{model.id}</option>
            {/each}
          </select>

          <button
            type="button"
            class={BUTTON}
            disabled={draft.asking !== null || draft.trying === ""}
            onclick={() => verify(draft)}
          >
            {t("settings-verify-key")}
          </button>
        </div>

        {@render answer(draft, "key")}

      </div>
    </article>
  {/if}
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

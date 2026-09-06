<script lang="ts">
  import { providerModels, verifyProvider, type Preset } from "../lib/ipc";
  import { t } from "../lib/i18n.svelte";
  import { saidBy } from "../lib/sending";
  import { edited, fresh, type Draft } from "./drafts";
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
    drafts.push(fresh());
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
</script>

<section class="flex flex-col gap-3">
  <div class="flex items-baseline justify-between gap-3">
    <h2 class="text-xs font-semibold tracking-wide uppercase opacity-50">
      {t("settings-providers")}
    </h2>
    <button type="button" class={BUTTON} onclick={add}>
      {t("settings-add-provider")}
    </button>
  </div>

  <!--
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
              <input
                bind:value={model.id}
                class="{FIELD} flex-1"
                autocorrect="off"
              />

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

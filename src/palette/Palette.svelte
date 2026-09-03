<script lang="ts">
  import { onMount, tick } from "svelte";
  import {
    actions as offeredActions,
    dismiss,
    lastCapture,
    onCapture,
    onCapturing,
    openAccessibility,
    run,
    status,
    type Action,
    type CaptureError,
    type CaptureOutcome,
    type Capturing,
  } from "../lib/ipc";
  import { t } from "../lib/i18n.svelte";
  import { latest } from "../lib/latest.svelte";
  import { sending } from "../lib/sending";

  const capture = latest<CaptureOutcome>({
    began: onCapturing,
    completed: onCapture,
    last: lastCapture,
  });

  /**
   * What a Capture on this desktop can read. Asked once: it is the session
   * Demysto was launched into, and a session does not change under it.
   */
  let capturing = $state<Capturing | null>(null);

  onMount(() => {
    const stop = capture.watch();

    status().then((reported) => (capturing = reported.capturing));

    return stop;
  });

  /**
   * The sentence a session that cannot read a Selection is owed, `null`
   * everywhere else (user story 56).
   *
   * Said here rather than carried from the backend: the sentence is a sentence
   * like any other, and the catalogue is where they live. What the backend says
   * is which kind of session this is.
   */
  const clipboardOnly = $derived(
    capturing?.reads === "clipboard_only" ? t("capture-clipboard-only") : null,
  );

  const outcome = $derived(capture.value);
  const captured = $derived(
    outcome?.status === "captured" ? outcome.detail : null,
  );
  const selection = $derived(
    captured && captured.origin !== "nothing" ? captured.selection : null,
  );

  /**
   * Where this Capture came from, in the words the Palette says it in — and
   * `null` where there is nothing to say, which is every state that produced
   * no Selection: those say what happened in a sentence of their own.
   */
  const origin = $derived(
    captured?.origin === "selection"
      ? t("palette-origin-selection")
      : captured?.origin === "clipboard"
        ? t("palette-origin-clipboard")
        : null,
  );

  /** The Actions that accept this Capture, as the backend filtered them. */
  let offered = $state<Action[]>([]);
  /** What the user has typed to narrow that list. */
  let filter = $state("");
  /** Which of the Actions still matching is the one Enter runs. */
  let highlighted = $state(0);
  /** The Action whose Parameters are being collected, once Enter chose one. */
  let collecting = $state<Action | null>(null);
  /** Which of that Action's Parameters is being asked for. */
  let asking = $state(0);
  /** What has been answered so far, by Parameter identifier. */
  let answers = $state<Record<string, string>>({});
  /** Whichever field is on screen: there is never more than one. */
  let field = $state<HTMLInputElement | null>(null);

  /** What stopped the user being sent somewhere, in the backend's own words. */
  let unreachable = $state<string | null>(null);

  /**
   * What a Capture that failed is owed, in a whole sentence.
   *
   * The backend reports which of the three it was and quotes whatever the
   * platform said; the sentence around that is the catalogue's, like every
   * other sentence in this window.
   */
  function refused(failure: CaptureError): string {
    switch (failure.kind) {
      case "clipboard":
        return t("capture-clipboard-unavailable", { detail: failure.message });
      case "keystroke":
        return t("capture-keystroke-refused", { detail: failure.message });
      case "permission":
        return t("capture-no-accessibility");
    }
  }

  /** Walks the user to the permission macOS is withholding (user story 55). */
  async function grant() {
    unreachable = await sending(openAccessibility);
  }

  /** What every button in this window looks like, which is the same. */
  const BUTTON =
    "cursor-pointer rounded border border-neutral-300 px-2 py-1 text-xs " +
    "hover:bg-neutral-100 dark:border-neutral-700 dark:hover:bg-neutral-800";

  /** What both of those fields look like, which is the same. */
  const FIELD =
    "w-full rounded border border-neutral-300 bg-transparent px-2 py-1 text-sm " +
    "outline-none focus:border-neutral-500 dark:border-neutral-700 " +
    "dark:focus:border-neutral-500";

  const matching = $derived(
    offered.filter((action) =>
      action.name.toLowerCase().includes(filter.trim().toLowerCase()),
    ),
  );

  // Clamped rather than corrected: a filter that leaves fewer Actions than the
  // one before it must not leave the highlight past the end of the list.
  const at = $derived(Math.min(highlighted, Math.max(matching.length - 1, 0)));
  const chosen = $derived(matching[at] ?? null);
  const parameter = $derived(collecting?.parameters[asking] ?? null);

  /**
   * Whether the Parameter on screen is the last one, which is the difference
   * between a button that runs the Action and one that asks the next question.
   */
  const final = $derived(
    collecting !== null && asking + 1 >= collecting.parameters.length,
  );

  // Every Capture starts the Palette over: it is hidden rather than unloaded,
  // so without this it comes back up filtered by what was typed into it last.
  $effect(() => {
    const current = outcome;

    filter = "";
    highlighted = 0;
    collecting = null;
    asking = 0;
    answers = {};

    if (current?.status !== "captured" || current.detail.origin === "nothing") {
      offered = [];
      return;
    }

    let stale = false;

    offeredActions().then((list) => {
      if (!stale) offered = list;
    });

    return () => {
      stale = true;
    };
  });

  // The field is focused as the Palette opens and again as it moves between
  // listing Actions and collecting a Parameter, because each is a different
  // element and the one before it has gone.
  $effect(() => {
    void outcome;
    void collecting;
    void asking;

    tick().then(() => field?.focus());
  });

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();

      // The way out of a question is the question before it: Escape steps
      // back through the Parameters, reaches the list of Actions from the
      // first of them, and only closes the Palette from there.
      if (collecting) {
        back();
      } else {
        dismiss();
      }

      return;
    }

    if (collecting) {
      if (event.key === "Enter") {
        event.preventDefault();
        answered();
      }

      return;
    }

    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      move(event.key === "ArrowDown" ? 1 : -1);

      return;
    }

    if (event.key === "Enter" && chosen) {
      event.preventDefault();
      choose(chosen);
    }
  }

  /** Moves the highlight, wrapping round rather than stopping at the ends. */
  function move(by: number) {
    if (matching.length === 0) return;

    highlighted = (at + by + matching.length) % matching.length;
  }

  /** Starts the Run, or the questions that have to be answered before it. */
  function choose(action: Action) {
    if (action.parameters.length === 0) {
      run(action.id, {});
      return;
    }

    collecting = action;
    asking = 0;
    answers = Object.fromEntries(
      action.parameters.map((parameter) => [parameter.id, parameter.default]),
    );
  }

  /**
   * Goes back to the question before this one, and to the list of Actions from
   * the first of them.
   *
   * What has been answered is kept: stepping back is how a mistyped first
   * answer is corrected, and clearing the rest would make it cost all of them.
   */
  function back() {
    if (asking > 0) {
      asking -= 1;
      return;
    }

    collecting = null;
  }

  /** Moves on to the next Parameter, or runs once there is none. */
  function answered() {
    if (!collecting) return;

    if (asking + 1 < collecting.parameters.length) {
      asking += 1;
      return;
    }

    run(collecting.id, answers);
  }
</script>

<svelte:window onkeydown={onKeydown} />

<main
  class="flex h-screen flex-col gap-3 border border-black/10 bg-white p-4 font-sans
         text-neutral-900 dark:border-white/10 dark:bg-neutral-900 dark:text-neutral-100"
>
  <header class="flex items-baseline justify-between gap-3">
    <h1 class="text-sm font-semibold tracking-tight">{t("app-name")}</h1>

    <!-- Drawn as quietly as before and now said as well. The role is what
         keeps it in the accessibility tree at all: WebKitGTK keeps no inline
         run that has neither a role nor a name, and this caption reached the
         tree nowhere. What carries the origin to a screen reader is the
         region below, named by this.

         That `status` is also a live region is a bonus and not the mechanism.
         The Capture reaches this window before the window is shown — see
         `palette.rs`, where the event is emitted ahead of `show` on purpose,
         so that the Palette never comes up holding the Capture before this
         one — and words that change while the window is hidden are announced
         by nobody. It is drawn empty rather than not at all so that the one
         path that does change them in front of somebody — the tray or a
         second launch re-opening a Palette already on screen — has a region
         to speak from (ticket 18). -->
    <span id="capture-origin" role="status" class="text-xs opacity-50"
      >{origin ?? ""}</span
    >
  </header>

  <div class="flex min-h-0 flex-1 flex-col gap-2 overflow-hidden">
    <!-- Named by that caption rather than headed by one of its own: reaching
         the captured text, a screen reader reads where it came from as part
         of it, and the window says nothing louder than it did. -->
    <section
      aria-labelledby={origin === null ? undefined : "capture-origin"}
      class="flex flex-col gap-2"
    >
      {#if outcome === null}
        <p class="text-sm opacity-50">
          {clipboardOnly
            ? t("palette-reading-clipboard")
            : t("palette-reading-selection")}
        </p>
      {:else if outcome.status === "failed"}
        <p class="text-sm text-red-600 dark:text-red-400">
          {refused(outcome.detail)}
        </p>

        {#if outcome.detail.kind === "permission"}
          <!-- The one Capture failure with somewhere to be sent: nothing about
               this Palette can fix it, and the pane that can is one click away. -->
          <div>
            <button
              type="button"
              onclick={grant}
              class={BUTTON}
            >
              {t("palette-open-accessibility")}
            </button>
          </div>

          {#if unreachable}
            <p class="text-xs text-red-600 dark:text-red-400">{unreachable}</p>
          {/if}
        {/if}
      {:else if !selection}
        {#if clipboardOnly}
          <!-- Not "nothing is selected": on this desktop a Selection was never
               something Demysto could have read, and saying so is the difference
               between a limitation and a tool that appears broken. -->
          <p class="text-sm opacity-60">{clipboardOnly}</p>
        {:else}
          <p class="text-sm opacity-60">{t("palette-nothing-captured")}</p>
        {/if}
      {:else}
        <p class="line-clamp-2 text-sm whitespace-pre-wrap opacity-60">
          {selection.text}
        </p>
      {/if}
    </section>

    <!--
      Both fields say `autocorrect="off"`, and that is not a matter of taste.
      macOS puts a candidate window over a WebKit field as soon as a word is
      typed into it, and while that window is up every Hotkey Demysto has
      claimed is held rather than delivered: the Hotkey stops closing the
      Palette it has just opened, and an Action's own Hotkey stops running it,
      until a key ends the input session — Escape — and the whole queue arrives
      at once (ticket 21). Neither field is prose: one filters a list of Actions
      by name, the other collects a Parameter.
    -->
    {#if selection}
      {#if collecting && parameter}
        <label class="flex flex-col gap-1">
          <span class="text-xs opacity-60">
            {collecting.name} · {parameter.label}
          </span>
          <input
            bind:this={field}
            bind:value={answers[parameter.id]}
            class={FIELD}
            autocorrect="off"
          />
        </label>

        <!-- The mouse's half of this step. Enter and Escape do the same and
             remain what the footer teaches: whoever reached here from the
             keyboard needs none of this, and whoever clicked an Action should
             not be handed back to the keyboard to run it (user story 66). -->
        <div class="flex justify-end gap-2">
          <button type="button" class={BUTTON} onclick={back}>
            {t("palette-back")}
          </button>

          <button type="button" class={BUTTON} onclick={answered}>
            {final ? t("palette-run") : t("palette-next")}
          </button>
        </div>
      {:else}
        <input
          bind:this={field}
          bind:value={filter}
          oninput={() => (highlighted = 0)}
          placeholder={t("palette-filter")}
          class={FIELD}
          autocorrect="off"
        />

        <ul class="min-h-0 flex-1 overflow-y-auto">
          {#each matching as action, index (action.id)}
            <li>
              <button
                type="button"
                onclick={() => choose(action)}
                onmouseenter={() => (highlighted = index)}
                class="w-full cursor-pointer rounded px-2 py-1 text-left text-sm
                       {index === at
                  ? 'bg-neutral-200 dark:bg-neutral-700'
                  : ''}"
              >
                {action.name}
              </button>
            </li>
          {:else}
            <li class="px-2 py-1 text-sm opacity-50">
              {t("palette-no-action-matches")}
            </li>
          {/each}
        </ul>
      {/if}
    {/if}
  </div>

  <footer class="text-xs opacity-40">
    {#if collecting}
      {t("palette-keys-collecting")}
    {:else if selection}
      {t("palette-keys-choosing")}
    {:else}
      {t("palette-keys-closing")}
    {/if}
  </footer>
</main>

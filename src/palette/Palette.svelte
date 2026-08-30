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
    type CaptureOutcome,
    type Capturing,
  } from "../lib/ipc";
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
   */
  const clipboardOnly = $derived(
    capturing?.reads === "clipboard_only" ? capturing.detail : null,
  );

  const outcome = $derived(capture.value);
  const captured = $derived(
    outcome?.status === "captured" ? outcome.detail : null,
  );
  const selection = $derived(
    captured && captured.origin !== "nothing" ? captured.selection : null,
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

  /** Walks the user to the permission macOS is withholding (user story 55). */
  async function grant() {
    unreachable = await sending(openAccessibility);
  }

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

    tick().then(() => field?.focus());
  });

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();

      // The way out of a question is the question before it: Escape goes back
      // to the list of Actions, and only closes the Palette from there.
      if (collecting) {
        collecting = null;
        asking = 0;
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
    <h1 class="text-sm font-semibold tracking-tight">Demysto</h1>

    {#if captured?.origin === "selection"}
      <span class="text-xs opacity-50">Selection</span>
    {:else if captured?.origin === "clipboard"}
      <span class="text-xs opacity-50">From the clipboard</span>
    {/if}
  </header>

  <div class="flex min-h-0 flex-1 flex-col gap-2 overflow-hidden">
    {#if outcome === null}
      <p class="text-sm opacity-50">
        {clipboardOnly ? "Reading the clipboard…" : "Reading what you selected…"}
      </p>
    {:else if outcome.status === "failed"}
      <p class="text-sm text-red-600 dark:text-red-400">
        {outcome.detail.message}
      </p>

      {#if outcome.detail.kind === "permission"}
        <!-- The one Capture failure with somewhere to be sent: nothing about
             this Palette can fix it, and the pane that can is one click away. -->
        <div>
          <button
            type="button"
            onclick={grant}
            class="cursor-pointer rounded border border-neutral-300 px-2 py-1 text-xs
                   hover:bg-neutral-100 dark:border-neutral-700 dark:hover:bg-neutral-800"
          >
            Open Accessibility settings
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
        <p class="text-sm opacity-60">
          Nothing is selected and the clipboard is empty. Select some text and
          press the Hotkey again.
        </p>
      {/if}
    {:else}
      <p class="line-clamp-2 text-sm whitespace-pre-wrap opacity-60">
        {selection.text}
      </p>

      {#if collecting && parameter}
        <label class="flex flex-col gap-1">
          <span class="text-xs opacity-60">
            {collecting.name} · {parameter.label}
          </span>
          <input
            bind:this={field}
            bind:value={answers[parameter.id]}
            class={FIELD}
          />
        </label>
      {:else}
        <input
          bind:this={field}
          bind:value={filter}
          oninput={() => (highlighted = 0)}
          placeholder="Filter Actions…"
          class={FIELD}
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
              No Action is called that.
            </li>
          {/each}
        </ul>
      {/if}
    {/if}
  </div>

  <footer class="text-xs opacity-40">
    {#if collecting}
      Enter to run · Esc to go back
    {:else if selection}
      ↑↓ to choose · Enter to run · Esc to close
    {:else}
      Esc to close
    {/if}
  </footer>
</main>

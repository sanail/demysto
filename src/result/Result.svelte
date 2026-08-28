<script lang="ts">
  import { onMount } from "svelte";
  import { copy } from "../lib/clipboard";
  import {
    dismiss,
    lastRun,
    onAnswered,
    onRunning,
    onStreaming,
    runningAction,
    type Action,
    type RunOutcome,
  } from "../lib/ipc";
  import { latest } from "../lib/latest.svelte";
  import { copyable, COPIED, render } from "../lib/markdown";

  /** How near the bottom counts as reading along with the answer. */
  const PINNED = 32;

  /** How long a copy button says it copied something. */
  const ACKNOWLEDGED = 1200;

  const run = latest<RunOutcome, string>({
    began: onRunning,
    progressed: onStreaming,
    completed: onAnswered,
    last: lastRun,
  });

  /**
   * The Action this window is heading its answer with.
   *
   * Asked for rather than carried on the events, because the answer is the
   * same question's however many hand-overs it arrives in; and asked again
   * whenever a Run begins, because by then it is a different question.
   */
  let action = $state<Action | null>(null);

  onMount(() => {
    const stop = run.watch();
    const running = onRunning(heading);

    heading();

    return () => {
      stop();
      running.then((unlisten) => unlisten());
    };
  });

  function heading() {
    runningAction().then((named) => (action = named));
  }

  const outcome = $derived(run.value);

  /**
   * The answer as it stands: the finished one where there is one, and otherwise
   * as much of it as has arrived.
   */
  const answer = $derived.by(() => {
    if (outcome === null) return run.progress;

    return outcome.status === "answered" ? outcome.detail : null;
  });

  const rendered = $derived(answer === null ? "" : render(answer));

  let reading = $state<HTMLElement>();
  let pinned = $state(true);
  let copied = $state(false);

  // An answer arriving pushes the view down with it, unless the user has
  // scrolled up to read something, in which case it stays where they put it.
  $effect(() => {
    void rendered;

    if (pinned && reading) {
      reading.scrollTop = reading.scrollHeight;
    }
  });

  function onScroll() {
    if (!reading) return;

    const below = reading.scrollHeight - reading.scrollTop - reading.clientHeight;
    pinned = below < PINNED;
  }

  /** Says a copy landed, and stops saying so a moment later. */
  function acknowledge(say: (landed: boolean) => void) {
    say(true);
    setTimeout(() => say(false), ACKNOWLEDGED);
  }

  async function copyAnswer() {
    if (answer === null) return;

    await copy(answer);
    acknowledge((landed) => (copied = landed));
  }

  /**
   * One handler for every code block, however many an answer turns out to have:
   * the blocks are Markdown's markup rather than this component's, so there is
   * nowhere in them to hang a handler of their own.
   */
  async function onAnswerClick(event: MouseEvent) {
    const clicked = event.target;
    if (!(clicked instanceof Element)) return;

    // A URL the Model wrote is not somewhere the user asked to be taken, and
    // following it would replace the answer with a web page. Left as text
    // until a later ticket has somewhere to send them.
    if (clicked.closest("a")) event.preventDefault();

    const block = copyable(clicked);
    if (!block) return;

    await copy(block.code);

    const label = block.button.textContent;
    acknowledge((landed) => (block.button.textContent = landed ? COPIED : label));
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      dismiss();
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<main
  class="flex h-screen flex-col gap-3 bg-white p-6 font-sans text-neutral-900
         dark:bg-neutral-900 dark:text-neutral-100"
>
  <header class="flex items-baseline justify-between gap-3">
    <h1 class="text-sm font-semibold tracking-tight">{action?.name ?? "Demysto"}</h1>

    {#if answer !== null}
      <button
        type="button"
        class="rounded border border-neutral-300 px-2 py-0.5 text-xs
               hover:bg-neutral-100 dark:border-neutral-700 dark:hover:bg-neutral-800"
        onclick={copyAnswer}
      >
        {copied ? COPIED : "Copy answer"}
      </button>
    {/if}
  </header>

  <!-- Ticket 06 puts the follow-up Turns underneath this. -->
  <div
    bind:this={reading}
    onscroll={onScroll}
    class="min-h-0 flex-1 overflow-y-auto"
  >
    {#if outcome !== null && outcome.status === "failed"}
      <p class="text-sm text-red-600 dark:text-red-400">
        {outcome.detail.message}
      </p>
    {:else if answer === null}
      <p class="text-sm opacity-50">Asking the Model…</p>
    {:else}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="answer select-text" onclick={onAnswerClick}>
        {@html rendered}
      </div>
    {/if}
  </div>

  <footer class="text-xs opacity-40">Esc to close</footer>
</main>

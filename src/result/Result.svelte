<script lang="ts">
  import { onMount } from "svelte";
  import {
    dismiss,
    lastRun,
    onAnswered,
    onRunning,
    type RunOutcome,
  } from "../lib/ipc";
  import { latest } from "../lib/latest.svelte";

  const run = latest<RunOutcome>({
    began: onRunning,
    completed: onAnswered,
    last: lastRun,
  });

  onMount(run.watch);

  const outcome = $derived(run.value);

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      dismiss();
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<main
  class="flex h-screen flex-col gap-4 bg-white p-6 font-sans text-neutral-900
         dark:bg-neutral-900 dark:text-neutral-100"
>
  <header class="flex items-baseline justify-between gap-3">
    <h1 class="text-sm font-semibold tracking-tight">Explain</h1>
  </header>

  <!-- Ticket 04 renders this as Markdown, as it streams; ticket 06 puts the
       follow-up Turns underneath it. -->
  <div class="min-h-0 flex-1 overflow-y-auto">
    {#if outcome === null}
      <p class="text-sm opacity-50">Asking the Model…</p>
    {:else if outcome.status === "failed"}
      <p class="text-sm text-red-600 dark:text-red-400">
        {outcome.detail.message}
      </p>
    {:else}
      <p class="text-sm leading-relaxed whitespace-pre-wrap select-text">
        {outcome.detail}
      </p>
    {/if}
  </div>

  <footer class="text-xs opacity-40">Esc to close</footer>
</main>

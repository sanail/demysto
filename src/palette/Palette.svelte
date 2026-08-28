<script lang="ts">
  import { onMount } from "svelte";
  import {
    dismiss,
    lastCapture,
    onCapture,
    onCapturing,
    run,
    type CaptureOutcome,
  } from "../lib/ipc";
  import { latest } from "../lib/latest.svelte";

  const capture = latest<CaptureOutcome>({
    began: onCapturing,
    completed: onCapture,
    last: lastCapture,
  });

  onMount(capture.watch);

  const outcome = $derived(capture.value);
  const captured = $derived(
    outcome?.status === "captured" ? outcome.detail : null,
  );
  const selection = $derived(
    captured && captured.origin !== "nothing" ? captured.selection : null,
  );

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      dismiss();
    }

    // Ticket 05 puts the Actions on screen and gives Enter one to run; until
    // then there is exactly one, and Enter runs it.
    if (event.key === "Enter" && selection) {
      event.preventDefault();
      run();
    }
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

  <div class="min-h-0 flex-1 overflow-hidden">
    {#if outcome === null}
      <p class="text-sm opacity-50">Reading what you selected…</p>
    {:else if outcome.status === "failed"}
      <p class="text-sm text-red-600 dark:text-red-400">
        {outcome.detail.message}
      </p>
    {:else if selection}
      <p class="line-clamp-6 text-sm whitespace-pre-wrap">{selection.text}</p>
    {:else}
      <p class="text-sm opacity-60">
        Nothing is selected and the clipboard is empty. Select some text and
        press the Hotkey again.
      </p>
    {/if}
  </div>

  <!-- Ticket 05 puts the Actions that accept this Selection here. -->
  <footer class="text-xs opacity-40">
    {selection ? "Enter to explain · Esc to close" : "Esc to close"}
  </footer>
</main>

<script lang="ts">
  import { status, type Status } from "./lib/ipc";

  // The skeleton window: it exists to prove the path from the core, through the
  // command bridge, to the screen. Ticket 02 replaces it with the Palette.
  const loading: Promise<Status> = status();
</script>

<main class="flex h-screen flex-col justify-center gap-2 px-8 font-sans">
  <h1 class="text-2xl font-semibold">Demysto</h1>

  {#await loading}
    <p class="text-sm opacity-60">Starting…</p>
  {:then reported}
    <p class="text-sm opacity-60">Version {reported.version}</p>
    <p class="font-mono text-xs break-all opacity-40">{reported.config_dir}</p>
  {:catch error}
    <p class="text-sm text-red-600">{error}</p>
  {/await}
</main>

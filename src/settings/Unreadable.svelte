<script lang="ts">
  import { t } from "../lib/i18n.svelte";

  let { said }: { /** Why the settings file could not be read. */ said: string } =
    $props();
</script>

<!-- Demysto will not write over a file it could not parse: whatever is in it,
     comments and keys alike, would go. So the panels backed by that file offer
     nothing but what is wrong and where.

     Said on each of them rather than once for the window, because the panels
     that are not backed by the file go on working: Actions are files of their
     own, the login items are the system's list, and the logs and the update
     check never touched the settings file at all. A window that put this in
     front of all four would be taking away three that still work. -->
<section class="flex flex-col gap-2">
  <p class="text-sm text-red-600 dark:text-red-400">{said}</p>
  <p class="text-sm opacity-60">{t("settings-unreadable-file")}</p>
</section>

<script lang="ts">
  import { onMount, tick } from "svelte";
  import { copy } from "../lib/clipboard";
  import {
    conversation,
    conversations,
    dismiss,
    followUp,
    onAnswered,
    onRunning,
    onStreaming,
    showConversation,
    stop,
    type Conversation,
    type Summary,
    type Turn,
  } from "../lib/ipc";
  import { copyable, COPIED, render } from "../lib/markdown";

  /** How near the bottom counts as reading along with the answer. */
  const PINNED = 32;

  /** How long a copy button says it copied something. */
  const ACKNOWLEDGED = 1200;

  /**
   * The Conversation on screen, as the core last had it.
   *
   * Asked for on every change rather than carried on the events, because the
   * window is opened by the backend and hidden rather than unloaded: what it
   * showed last time is still on screen when it is next shown, and the Turn it
   * is about to show may have begun before it ever loaded. One question answers
   * both, whatever it missed.
   */
  let showing = $state<Conversation | null>(null);

  /** How far the Turn under way has got, or `null` when it has not said. */
  let progress = $state<string | null>(null);

  /**
   * Which question the answer on screen came back to, so that an older one
   * resolving late cannot put the window behind a newer one.
   */
  let asked = 0;

  onMount(() => {
    const listening = [
      onRunning(() => {
        // What the last Turn streamed is not this one's, and the Conversation
        // this one is in may not be the Conversation that was on screen.
        progress = null;
        refresh();
      }),
      onAnswered(refresh),
      onStreaming((answer) => (progress = answer)),
    ];

    refresh();
    tick().then(() => composer?.focus());

    return () => listening.forEach((listener) => listener.then((off) => off()));
  });

  /** Asks for the Conversation as the core now has it, and shows that. */
  async function refresh() {
    await show(conversation());
  }

  /**
   * Puts whatever a question about a Conversation answers with on screen,
   * unless a later question has already been answered: two events in quick
   * succession are two questions, and the older one resolving last would leave
   * the window a Turn behind.
   */
  async function show(asking: Promise<Conversation | null>) {
    const mine = ++asked;
    const next = await asking;

    if (mine === asked) showing = next;
  }

  const turns = $derived(showing?.turns ?? []);

  /** Whether the Model is still answering, which is what Stop is for. */
  const answering = $derived(turns.at(-1)?.outcome === null);

  /**
   * What one Turn is showing: what it produced, or — for the Turn still being
   * answered — as much of it as has arrived. `null` while there is neither.
   */
  function said(turn: Turn, last: boolean): string | null {
    if (turn.outcome === null) return last ? progress : null;

    return turn.outcome.status === "failed" ? null : turn.outcome.detail;
  }

  let reading = $state<HTMLElement>();
  let composer = $state<HTMLTextAreaElement>();
  let pinned = $state(true);

  /** Which Turn's copy button is saying it copied something. */
  let copied = $state<number | null>(null);

  /** What the user has typed but not yet asked. */
  let question = $state("");

  /** Whether the list of this session's Conversations is open, and what it holds. */
  let listing = $state(false);
  let list = $state<Summary[]>([]);

  // An answer arriving pushes the view down with it, unless the user has
  // scrolled up to read something, in which case it stays where they put it.
  $effect(() => {
    void turns;
    void progress;

    if (pinned && reading) {
      reading.scrollTop = reading.scrollHeight;
    }
  });

  function onScroll() {
    if (!reading) return;

    const below = reading.scrollHeight - reading.scrollTop - reading.clientHeight;
    pinned = below < PINNED;
  }

  /** Asks what has been typed as a follow-up Turn in this Conversation. */
  async function ask() {
    const asking = question.trim();
    if (asking === "" || answering) return;

    // Cleared before the Turn rather than after it: the question is on screen
    // from the moment the backend has it, and the field is where the next one
    // is typed.
    question = "";
    pinned = true;

    await followUp(asking);
  }

  /** Opens the list of this session's Conversations, or closes it again. */
  async function toggle() {
    listing = !listing;

    if (listing) list = await conversations();
  }

  /** Goes back to an earlier Conversation, which is then the one asked in. */
  async function goBackTo(id: number) {
    listing = false;
    progress = null;

    await show(showConversation(id));
  }

  /** Says a copy landed, and stops saying so a moment later. */
  function acknowledge(say: (landed: boolean) => void) {
    say(true);
    setTimeout(() => say(false), ACKNOWLEDGED);
  }

  async function copyAnswer(at: number, answer: string) {
    await copy(answer);
    acknowledge((landed) => (copied = landed ? at : null));
  }

  /**
   * One handler for every code block, however many the Turns turn out to have:
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
    if (event.key !== "Escape") return;

    event.preventDefault();

    // The way out of a list is the list: Escape closes it, and only closes the
    // window from there.
    if (listing) {
      listing = false;
    } else {
      dismiss();
    }
  }

  function onComposerKeydown(event: KeyboardEvent) {
    // Enter asks, because asking is what the field is for; a new line inside a
    // question is the rarer thing and keeps the modifier.
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      ask();
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<main
  class="flex h-screen flex-col gap-3 bg-white p-6 font-sans text-neutral-900
         dark:bg-neutral-900 dark:text-neutral-100"
>
  <header class="relative flex items-baseline justify-between gap-3">
    <h1 class="text-sm font-semibold tracking-tight">
      {showing?.action?.name ?? "Demysto"}
    </h1>

    <button
      type="button"
      class="rounded border border-neutral-300 px-2 py-0.5 text-xs
             hover:bg-neutral-100 dark:border-neutral-700 dark:hover:bg-neutral-800"
      onclick={toggle}
    >
      Conversations
    </button>

    {#if listing}
      <ul
        class="absolute right-0 top-6 z-10 max-h-64 w-80 overflow-y-auto rounded border
               border-neutral-300 bg-white py-1 text-xs shadow-lg
               dark:border-neutral-700 dark:bg-neutral-800"
      >
        {#each list as held (held.id)}
          <li>
            <button
              type="button"
              class="flex w-full flex-col items-start gap-0.5 px-3 py-1.5 text-left
                     hover:bg-neutral-100 dark:hover:bg-neutral-700"
              class:font-semibold={held.id === showing?.id}
              onclick={() => goBackTo(held.id)}
            >
              <span>{held.name ?? "Conversation"}</span>
              {#if held.about !== ""}
                <span class="w-full truncate opacity-50">{held.about}</span>
              {/if}
            </button>
          </li>
        {:else}
          <li class="px-3 py-1.5 opacity-50">Nothing asked yet.</li>
        {/each}
      </ul>
    {/if}
  </header>

  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    bind:this={reading}
    onscroll={onScroll}
    onclick={onAnswerClick}
    class="flex min-h-0 flex-1 flex-col gap-6 overflow-y-auto"
  >
    {#each turns as turn, at (at)}
      {@const last = at === turns.length - 1}
      {@const text = said(turn, last)}

      <article class="flex flex-col gap-2">
        {#if turn.question !== null}
          <p
            class="max-w-[85%] self-end whitespace-pre-wrap rounded-lg bg-neutral-100
                   px-3 py-1.5 text-sm select-text dark:bg-neutral-800"
          >
            {turn.question}
          </p>
        {/if}

        {#if turn.outcome?.status === "failed"}
          <p class="text-sm text-red-600 dark:text-red-400">
            {turn.outcome.detail.message}
          </p>
        {:else if text === null}
          <p class="text-sm opacity-50">Asking the Model…</p>
        {:else}
          {#if text !== ""}
            <div class="answer select-text">
              {@html render(text)}
            </div>
          {/if}

          {#if turn.outcome !== null}
            <div class="flex items-center gap-3 text-xs opacity-40">
              {#if text !== ""}
                <button
                  type="button"
                  class="cursor-pointer rounded px-1 py-0.5 hover:bg-neutral-100
                         dark:hover:bg-neutral-800"
                  onclick={() => copyAnswer(at, text)}
                >
                  {copied === at ? COPIED : "Copy answer"}
                </button>
              {/if}

              {#if turn.outcome.status === "stopped"}
                <span>Stopped</span>
              {/if}
            </div>
          {/if}
        {/if}
      </article>
    {/each}
  </div>

  <footer class="flex flex-col gap-1.5">
    <div class="flex items-end gap-2">
      <textarea
        bind:this={composer}
        bind:value={question}
        onkeydown={onComposerKeydown}
        rows="1"
        placeholder="Ask a follow-up…"
        class="max-h-32 min-h-8 w-full flex-1 resize-none rounded border border-neutral-300
               bg-transparent px-2 py-1 text-sm outline-none focus:border-neutral-500
               dark:border-neutral-700 dark:focus:border-neutral-500"
      ></textarea>

      {#if answering}
        <button
          type="button"
          class="rounded border border-neutral-300 px-2 py-1 text-xs
                 hover:bg-neutral-100 dark:border-neutral-700 dark:hover:bg-neutral-800"
          onclick={stop}
        >
          Stop
        </button>
      {:else}
        <button
          type="button"
          class="rounded border border-neutral-300 px-2 py-1 text-xs
                 hover:bg-neutral-100 disabled:opacity-40 disabled:hover:bg-transparent
                 dark:border-neutral-700 dark:hover:bg-neutral-800
                 dark:disabled:hover:bg-transparent"
          disabled={question.trim() === ""}
          onclick={ask}
        >
          Ask
        </button>
      {/if}
    </div>

    <p class="text-xs opacity-40">
      Enter to ask, Shift+Enter for a new line, Esc to close
    </p>
  </footer>
</main>

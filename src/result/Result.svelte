<script lang="ts">
  import { onMount, tick } from "svelte";
  import { copy } from "../lib/clipboard";
  import {
    continueAnswer,
    conversation,
    conversations,
    dismiss,
    followUp,
    models as configuredModels,
    onAnswered,
    onRunning,
    onStreaming,
    openAccessibility,
    openSettings,
    retry,
    selection,
    showConversation,
    stop,
    type Conversation,
    type RunError,
    type RunOutcome,
    type Summary,
    type Turn,
  } from "../lib/ipc";
  import { t } from "../lib/i18n.svelte";
  import { copyable, render } from "../lib/markdown";
  import { sending } from "../lib/sending";

  /** How near the bottom counts as reading along with the answer. */
  const PINNED = 32;

  /** How long a copy button says it copied something. */
  const ACKNOWLEDGED = 1200;

  /** What every control under an answer looks like. */
  const FOOTER =
    "cursor-pointer rounded border border-neutral-300 bg-transparent px-1.5 py-0.5 " +
    "hover:bg-neutral-100 dark:border-neutral-700 dark:hover:bg-neutral-800";

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
        //
        // Except where the Turn is being carried on: that one is adding to
        // what is on screen, and clearing it would blank the answer for as
        // long as the Model takes to say the next word.
        progress = carryingOn ? progress : null;
        carryingOn = false;
        refresh();
      }),
      onAnswered(() => {
        // Cleared here rather than only where it is set: a continuation the
        // backend declines never reaches `onRunning`, and a flag left standing
        // would keep the next Turn showing the last one's text until its first
        // hand-over arrived. Every Turn ends here, however it ended.
        carryingOn = false;
        refresh();
      }),
      onStreaming((answer) => (progress = answer)),
    ];

    refresh();
    tick().then(() => composer?.focus());

    return () => listening.forEach((listener) => listener.then((off) => off()));
  });

  /** Asks for the Conversation as the core now has it, and shows that. */
  async function refresh() {
    await show(conversation());

    // Fetched the moment a failure is on screen rather than when the field is
    // opened: a native select builds its menu at the click, before an answer
    // from the backend could arrive, so a list asked for on focus is a list
    // that is empty exactly when somebody wants it. Asked again each time,
    // because Settings may have added a Model since — this window stays open
    // for the session.
    if (wrong(turns.at(-1)?.outcome ?? null)) {
      switchable = await configuredModels();
    }
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
   *
   * A Turn that broke off part-way shows what did arrive: that text is the
   * user's answer as far as it got, and the error under it says why there is
   * no more of it yet.
   */
  function said(turn: Turn, last: boolean): string | null {
    if (turn.outcome === null) return last ? progress : null;

    switch (turn.outcome.status) {
      case "failed":
        return null;
      case "interrupted":
        return turn.outcome.detail.text;
      default:
        return turn.outcome.detail;
    }
  }

  /** Why a Turn has no more to show, where something went wrong. */
  function wrong(outcome: RunOutcome | null): RunError | null {
    if (outcome === null) return null;

    switch (outcome.status) {
      case "failed":
        return outcome.detail;
      case "interrupted":
        return outcome.detail.error;
      default:
        return null;
    }
  }

  let reading = $state<HTMLElement>();
  let composer = $state<HTMLTextAreaElement>();
  let pinned = $state(true);

  /** Which Turn's copy button is saying it copied something. */
  let copied = $state<number | null>(null);

  /** What the user has typed but not yet asked. */
  let question = $state("");

  /**
   * Every Model configured, by the name a Conversation is switched to. Filled
   * in by [`refresh`], which is the only place that knows a failure has just
   * landed.
   */
  let switchable = $state<string[]>([]);

  /**
   * Whether the Turn about to begin is carrying on the one on screen, so that
   * what already arrived is not blanked while the rest of it is asked for.
   */
  let carryingOn = $state(false);

  /**
   * What stopped the user being sent somewhere, in the backend's own words. One
   * field for both buttons below: they hang off the same failure's footer, and
   * only one of them is ever on screen.
   */
  let unreachable = $state<string | null>(null);

  /** The quotation of the Selection, and whether all of it is on screen. */
  let quotation = $state<HTMLElement>();
  let expanded = $state(false);

  /** The whole Selection, once it has been asked for; `null` until then. */
  let whole = $state<string | null>(null);

  /**
   * Whether the quotation has more in it than the lines it is showing, which is
   * what puts Show more under it.
   *
   * Measured rather than worked out from the preview's length: the preview is
   * capped well beyond two lines, so a Selection far shorter than the cap is
   * still the common case for being clipped, and only the layout knows.
   */
  let clipped = $state(false);

  /** Which Conversation the quotation belongs to, so that going to another closes it. */
  let quoting: number | null = null;

  // A different Conversation is a different Selection: the quotation opens
  // closed again rather than standing expanded on somebody else's text.
  $effect(() => {
    const id = showing?.id ?? null;
    if (id === quoting) return;

    quoting = id;
    expanded = false;
    whole = null;
  });

  /** Says whether the quotation is showing less than it holds. */
  function measure() {
    clipped =
      quotation !== undefined &&
      !expanded &&
      quotation.scrollHeight > quotation.clientHeight;
  }

  // Re-measured whenever what is quoted or how much of it changes. The `void`s
  // are the dependencies: nothing here reads them, and the measurement is of
  // the DOM they produced.
  //
  // After a `tick` rather than in the effect itself, because collapsing and
  // measuring are one change: the Conversation being switched puts the
  // quotation back to two lines, and a measurement taken in the same breath
  // finds the box it had while expanded, where nothing is clipped and Show
  // more never appears.
  $effect(() => {
    void showing?.preview;
    void whole;
    void expanded;

    tick().then(measure);
  });

  /** Shows the whole Selection, asking for the part the preview left out. */
  async function expand() {
    expanded = true;

    // Asked once per Conversation: the Selection cannot change under a
    // Conversation, so showing it a second time is showing what is already here.
    if (whole === null) whole = await selection();
  }

  /** Walks the user to the permission macOS is withholding (user story 55). */
  async function grant() {
    unreachable = await sending(openAccessibility);
  }

  /** Asks the Turn on screen again, of the Model the user picked or of the same one. */
  async function askAgain(model?: string) {
    pinned = true;
    progress = null;

    await retry(model);
  }

  /** Asks the Model for the rest of an answer that broke off part-way. */
  async function carryOn() {
    pinned = true;
    carryingOn = true;

    await continueAnswer();
  }

  /** Opens Settings where the Provider that refused a key is configured. */
  async function fix(provider: string) {
    unreachable = await sending(() => openSettings(provider));
  }

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
    acknowledge((landed) => {
      block.button.textContent = landed ? t("code-copied") : label;
    });
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

<svelte:window onkeydown={onKeydown} onresize={measure} />

<main
  class="flex h-screen flex-col gap-3 bg-white p-6 font-sans text-neutral-900
         dark:bg-neutral-900 dark:text-neutral-100"
>
  <header class="relative flex items-baseline justify-between gap-3">
    <h1 class="text-sm font-semibold tracking-tight">
      {showing?.action?.name ?? t("app-name")}
    </h1>

    <button
      type="button"
      class="rounded border border-neutral-300 px-2 py-0.5 text-xs
             hover:bg-neutral-100 dark:border-neutral-700 dark:hover:bg-neutral-800"
      onclick={toggle}
    >
      {t("result-conversations")}
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
              <span>{held.name ?? t("result-conversation-unnamed")}</span>
              {#if held.about !== ""}
                <span class="w-full truncate opacity-50">{held.about}</span>
              {/if}
            </button>
          </li>
        {:else}
          <li class="px-3 py-1.5 opacity-50">{t("result-nothing-asked-yet")}</li>
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
    {#if showing?.preview}
      <!-- What the Model is being asked about, quoted above what it said: the
           window opens over whatever the user was reading, and the Action's
           name alone does not say which paragraph it was handed — nor whether
           the Capture took the right one.

           Not headed with where it came from. The Conversation does not carry
           that, and a heading reading "Selection" would be untrue of the Runs
           that fell back to the clipboard.

           What it is, though, has to be said rather than drawn: on screen the
           rule down the left and the grey say "this is the text, not the
           answer", and a screen reader is given neither. So the quotation is a
           named region — true of both origins, and nothing added to the window
           (ticket 18). -->
      <section
        aria-label={t("result-quotation-label")}
        class="flex flex-col items-start gap-1.5"
      >
        <blockquote
          bind:this={quotation}
          class="border-l-2 border-neutral-200 pl-3 text-sm whitespace-pre-wrap
                 opacity-60 select-text dark:border-neutral-700"
          class:line-clamp-2={!expanded}
        >
          {expanded ? (whole ?? showing.preview) : showing.preview}
        </blockquote>

        {#if clipped || expanded}
          <button
            type="button"
            class="{FOOTER} opacity-40"
            onclick={() => (expanded ? (expanded = false) : expand())}
          >
            {expanded ? t("result-show-less") : t("result-show-more")}
          </button>
        {/if}
      </section>
    {/if}

    {#if showing?.warning}
      <!-- Said before the answer and left there: the Selection is what every
           Turn in this Conversation is about, so this is about all of them. -->
      <p
        class="rounded border border-amber-300 bg-amber-50 px-3 py-2 text-xs
               text-amber-900 dark:border-amber-900 dark:bg-amber-950
               dark:text-amber-200"
      >
        {showing.warning}
      </p>
    {/if}

    {#each turns as turn, at (at)}
      {@const last = at === turns.length - 1}
      {@const text = said(turn, last)}
      {@const problem = wrong(turn.outcome)}

      <article class="flex flex-col gap-2">
        {#if turn.question !== null}
          <p
            class="max-w-[85%] self-end whitespace-pre-wrap rounded-lg bg-neutral-100
                   px-3 py-1.5 text-sm select-text dark:bg-neutral-800"
          >
            {turn.question}
          </p>
        {/if}

        {#if text === null && problem === null}
          <p class="text-sm opacity-50">{t("result-asking")}</p>
        {:else}
          {#if text !== null && text !== ""}
            <div class="answer select-text">
              {@html render(text)}
            </div>
          {/if}

          {#if problem}
            <!-- Inside the Conversation and never as a dialog: the user asked a
                 question and is owed an answer to it, even when the answer is
                 what went wrong. -->
            <p class="text-sm text-red-600 select-text dark:text-red-400">
              {problem.message}
            </p>
          {/if}

          {#if turn.outcome !== null}
            <div class="flex flex-wrap items-center gap-3 text-xs">
              {#if text !== null && text !== ""}
                <button
                  type="button"
                  class="{FOOTER} opacity-40"
                  onclick={() => copyAnswer(at, text)}
                >
                  {copied === at ? t("result-copied") : t("result-copy-answer")}
                </button>
              {/if}

              {#if turn.outcome.status === "stopped"}
                <!-- With a role for the reason the Palette's origin caption
                     has one: WebKit keeps no inline run that has neither a
                     role nor a name, and "Stopped" is the whole of what says
                     this answer is short because the user said so.

                     `status` rather than a role that only names it, because
                     the moment this appears is the moment the user pressed
                     Stop and heard the answer go quiet: that is worth saying.
                     The price is that going back to another Conversation
                     whose last Turn was stopped can say the word again — true
                     of what is then on screen, which is why it is a price
                     worth paying rather than a bug. -->
                <span role="status" class="opacity-40">{t("result-stopped")}</span>
              {/if}

              <!-- Offered on the last Turn alone: what is asked again is the
                   Turn the Conversation ends on, and a button on an older one
                   would quietly act somewhere else. -->
              {#if last && problem}
                {#if turn.outcome.status === "interrupted"}
                  <button type="button" class={FOOTER} onclick={carryOn}>
                    {t("result-continue")}
                  </button>
                {/if}

                <!-- Not offered on a permission: asking again and asking
                     elsewhere both put a question to a Model, and no Model is
                     what this Turn is short of. Two buttons that cannot work
                     would report it as an ordinary failure with a third. -->
                {#if problem.kind !== "permission"}
                  <button type="button" class={FOOTER} onclick={() => askAgain()}>
                    {t("result-try-again")}
                  </button>

                  <select
                    class="{FOOTER} max-w-40"
                    value=""
                    onchange={(event) => {
                      const picked = event.currentTarget.value;
                      event.currentTarget.value = "";
                      if (picked !== "") askAgain(picked);
                    }}
                  >
                    <option value="">{t("result-ask-another-model")}</option>
                    {#each switchable as model (model)}
                      <option value={model}>{model}</option>
                    {/each}
                  </select>
                {/if}

                {#if problem.kind === "permission"}
                  <!-- The Run an Action's own Hotkey started, with no Palette
                       anywhere on the path to have reported this: the
                       Conversation is where the permission has to be said, and
                       where the way to it has to be offered. -->
                  <button type="button" class={FOOTER} onclick={grant}>
                    {t("result-open-accessibility")}
                  </button>
                {/if}

                {#if problem.kind === "authentication"}
                  <!-- The one failure the Conversation cannot fix: the key is
                       wrong, and the fix is in that Provider's own settings. -->
                  <button
                    type="button"
                    class={FOOTER}
                    onclick={() => fix(problem.provider)}
                  >
                    {t("result-open-provider-settings", {
                      provider: problem.provider,
                    })}
                  </button>
                {/if}
              {/if}
            </div>

            {#if last && unreachable}
              <p class="text-xs text-red-600 dark:text-red-400">
                {unreachable}
              </p>
            {/if}
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
        placeholder={t("result-follow-up")}
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
          {t("result-stop")}
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
          {t("result-ask")}
        </button>
      {/if}
    </div>

    <p class="text-xs opacity-40">{t("result-keys")}</p>
  </footer>
</main>

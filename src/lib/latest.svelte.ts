import type { UnlistenFn } from "@tauri-apps/api/event";

/** The work a window shows the state of, in the ways that state arrives. */
export type Work<T, P> = {
  /** Every start from here on, each of which discards what came before it. */
  began: (handle: () => void) => Promise<UnlistenFn>;
  /** How far along that work is, for work that says so as it goes. */
  progressed?: (handle: (progress: P) => void) => Promise<UnlistenFn>;
  /** Everything that work produced, from here on. */
  completed: (handle: (value: T) => void) => Promise<UnlistenFn>;
  /** What it produced last, for a window that arrived after it. */
  last: () => Promise<T | null>;
};

/**
 * What the backend last said, kept current.
 *
 * Both of Demysto's windows have the same shape of problem. They are opened by
 * the backend and hidden rather than unloaded, so what one showed last time is
 * still on screen when it is next shown; and the work whose state they show
 * begins before they are visible, so the events carrying it can arrive before
 * the window has ever loaded. Each therefore listens for the state changing and
 * asks once for whatever it missed — but only while it has heard nothing, since
 * by the time that question is answered a newer state may already have arrived.
 *
 * `null` is "there is nothing to show yet", which the window renders as the
 * work being under way: it is what both the beginning of a Run and the
 * beginning of a Capture leave behind.
 */
export function latest<T, P = never>(work: Work<T, P>) {
  let value = $state<T | null>(null);
  let progress = $state<P | null>(null);
  let heard = false;

  return {
    get value() {
      return value;
    },

    /** How far the work under way has got, or `null` when it has not said. */
    get progress() {
      return progress;
    },

    /** Starts listening, and answers with the function that stops. */
    watch() {
      const began = work.began(() => {
        heard = true;
        value = null;
        progress = null;
      });

      // Progress counts as having heard from the backend: what the question
      // below would answer with is older than what has just arrived.
      const progressed = work.progressed?.((next) => {
        heard = true;
        progress = next;
      });

      const completed = work.completed((next) => {
        heard = true;
        value = next;
      });

      work.last().then((missed) => {
        if (!heard) {
          value = missed;
        }
      });

      return () => {
        for (const listening of [began, progressed, completed]) {
          listening?.then((stop) => stop());
        }
      };
    },
  };
}

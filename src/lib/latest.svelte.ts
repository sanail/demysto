import type { UnlistenFn } from "@tauri-apps/api/event";

/** The work a window shows the state of, in the three ways that state arrives. */
export type Work<T> = {
  /** Every start from here on, each of which discards what came before it. */
  began: (handle: () => void) => Promise<UnlistenFn>;
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
export function latest<T>(work: Work<T>) {
  let value = $state<T | null>(null);
  let heard = false;

  return {
    get value() {
      return value;
    },

    /** Starts listening, and answers with the function that stops. */
    watch() {
      const began = work.began(() => {
        heard = true;
        value = null;
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
        began.then((stop) => stop());
        completed.then((stop) => stop());
      };
    },
  };
}

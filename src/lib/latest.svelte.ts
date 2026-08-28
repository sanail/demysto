import type { UnlistenFn } from "@tauri-apps/api/event";

/** The work a window shows the state of, in the ways that state arrives. */
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
 * The Palette is opened by the backend and hidden rather than unloaded, so what
 * it showed last time is still on screen when it is next shown; and the Capture
 * it shows begins before it is visible, so the events carrying one can arrive
 * before the window has ever loaded. It therefore listens for the state
 * changing and asks once for whatever it missed — but only while it has heard
 * nothing, since by the time that question is answered a newer state may
 * already have arrived.
 *
 * `null` is "there is nothing to show yet", which the window renders as the
 * work being under way: it is what the beginning of a Capture leaves behind.
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
        for (const listening of [began, completed]) {
          listening.then((stop) => stop());
        }
      };
    },
  };
}

/**
 * The one gesture every button that leaves this window makes.
 *
 * Settings, a system settings pane, a file manager: each of them either takes
 * the user somewhere, or leaves them here owed a sentence saying why not. The
 * shape is shared so that the buttons cannot drift into different apologies for
 * the same thing.
 */

/** The sentence a rejected command carries — every one of them has one. */
export function saidBy(error: unknown): string {
  if (typeof error === "object" && error !== null && "message" in error) {
    return String((error as { message: unknown }).message);
  }

  return String(error);
}

/**
 * Runs something that sends the user out of this window, and answers with what
 * stopped it in the backend's own words — `null` where nothing did.
 */
export async function sending(
  somewhere: () => Promise<void>,
): Promise<string | null> {
  try {
    await somewhere();
    return null;
  } catch (error) {
    return saidBy(error);
  }
}

/**
 * Puts text on the clipboard.
 *
 * The webview's own clipboard rather than the core's: what is being copied is
 * text this window already holds, and sending it back across the bridge so that
 * Rust could hand it to the same desktop would be a round trip for nothing.
 *
 * The fallback is for webviews that do not treat the application's own origin
 * as a secure context, where `navigator.clipboard` is not merely refused but
 * absent. `execCommand` is deprecated and implemented everywhere, which is the
 * combination that makes it a fallback rather than a first choice.
 */
export async function copy(text: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(text);
    return;
  } catch {
    // Fall through: an absent or refused clipboard API is not worth reporting
    // when there is another way to do the same thing.
  }

  const carrier = document.createElement("textarea");
  carrier.value = text;
  carrier.setAttribute("readonly", "");
  carrier.style.position = "fixed";
  carrier.style.top = "0";
  carrier.style.opacity = "0";

  document.body.append(carrier);
  carrier.select();
  document.execCommand("copy");
  carrier.remove();
}

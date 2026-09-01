import { mount } from "svelte";
import Result from "./Result.svelte";
import { speak } from "../lib/i18n.svelte";
import "../app.css";

// Mounted once the language is known, so that nothing is drawn in English and
// then redrawn: the backend has already settled it, and this asks which. The
// window is created hidden at startup and shown much later, so the round trip
// costs nobody a frame.
//
// And mounted even when that fails. A backend that could not say which language
// leaves English standing, which is what the catalogue starts in; a window that
// never mounted at all would be a blank one, and for the Palette — drawn on a
// Hotkey — that is indistinguishable from a Hotkey that does nothing.
speak()
  .catch(() => {})
  .then(() =>
    mount(Result, {
      target: document.getElementById("result")!,
    }),
  );

import { mount } from "svelte";
import Welcome from "./Welcome.svelte";
import { speak } from "../lib/i18n.svelte";
import "../app.css";

// Mounted once the language is known, for the reason the other windows are —
// and here it is the whole of the first step: the flow opens by showing the
// language Demysto found, and one drawn in English first would be showing the
// wrong answer to its own question.
speak()
  .catch(() => {})
  .then(() =>
    mount(Welcome, {
      target: document.getElementById("welcome")!,
    }),
  );

import { mount } from "svelte";
import Settings from "./Settings.svelte";
import "../app.css";

export default mount(Settings, {
  target: document.getElementById("settings")!,
});

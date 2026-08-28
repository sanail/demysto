import { mount } from "svelte";
import Palette from "./Palette.svelte";
import "../app.css";

export default mount(Palette, {
  target: document.getElementById("palette")!,
});

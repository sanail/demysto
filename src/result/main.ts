import { mount } from "svelte";
import Result from "./Result.svelte";
import "../app.css";

export default mount(Result, {
  target: document.getElementById("result")!,
});

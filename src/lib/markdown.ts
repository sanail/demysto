import MarkdownIt from "markdown-it";
import hljs from "highlight.js/lib/core";
import bash from "highlight.js/lib/languages/bash";
import c from "highlight.js/lib/languages/c";
import cpp from "highlight.js/lib/languages/cpp";
import csharp from "highlight.js/lib/languages/csharp";
import css from "highlight.js/lib/languages/css";
import diff from "highlight.js/lib/languages/diff";
import go from "highlight.js/lib/languages/go";
import java from "highlight.js/lib/languages/java";
import javascript from "highlight.js/lib/languages/javascript";
import json from "highlight.js/lib/languages/json";
import kotlin from "highlight.js/lib/languages/kotlin";
import markdown from "highlight.js/lib/languages/markdown";
import php from "highlight.js/lib/languages/php";
import python from "highlight.js/lib/languages/python";
import ruby from "highlight.js/lib/languages/ruby";
import rust from "highlight.js/lib/languages/rust";
import sql from "highlight.js/lib/languages/sql";
import swift from "highlight.js/lib/languages/swift";
import toml from "highlight.js/lib/languages/ini";
import typescript from "highlight.js/lib/languages/typescript";
import xml from "highlight.js/lib/languages/xml";
import yaml from "highlight.js/lib/languages/yaml";

/**
 * The languages code blocks are highlighted in.
 *
 * A chosen set rather than everything highlight.js knows: the full grammar
 * collection is an order of magnitude larger than the rest of this window put
 * together, and a window that has to appear instantly cannot afford to carry
 * grammars for languages nobody reading this is writing. See the spec's
 * *Shape*.
 */
const LANGUAGES = {
  bash,
  c,
  cpp,
  csharp,
  css,
  diff,
  go,
  java,
  javascript,
  json,
  kotlin,
  markdown,
  php,
  python,
  ruby,
  rust,
  sql,
  swift,
  toml,
  typescript,
  xml,
  yaml,
};

for (const [name, language] of Object.entries(LANGUAGES)) {
  hljs.registerLanguage(name, language);
}

/** The class a rendered code block carries, and the marker its copy button
 * carries. Both are private to this module: what a click on one means is
 * [`copyable`]'s to answer, because this is where the markup was written. */
const BLOCK = "md-code";
const COPY = "data-copy";

/** What a code block's copy button says once it has been pressed. */
export const COPIED = "Copied";

const md = MarkdownIt({
  // Model output is untrusted content, and this window renders it beside the
  // user's own selection. Raw HTML in it is text, not markup (the spec's
  // *Shape*). The CSP in `tauri.conf.json` is the second half of the same
  // argument: nothing in an answer reaches the network.
  html: false,
  // A URL the Model wrote is not one the user asked to visit, and this window
  // has nowhere to send them anyway — see `Result.svelte`, which stops the
  // click. Left off so that bare URLs at least stay selectable text.
  linkify: false,
  breaks: false,
});

/**
 * Code blocks, highlighted and carrying a button that copies them.
 *
 * Written out rather than left to markdown-it's own fence renderer because the
 * button belongs inside the block's own box, and because what is escaped and
 * what is not is worth having in one visible place.
 */
md.renderer.rules.fence = (tokens, index) => {
  const token = tokens[index];
  const language = token.info.trim().split(/\s+/)[0] ?? "";
  const code = highlighted(token.content, language);
  const label = language ? md.utils.escapeHtml(language) : "";

  return `<div class="${BLOCK}"><div class="${BLOCK}-bar"><span>${label}</span>\
<button type="button" ${COPY}>Copy</button></div>\
<pre><code class="hljs">${code}</code></pre></div>\n`;
};

/** One code block, marked up by highlight.js or merely made safe. */
function highlighted(code: string, language: string): string {
  if (hljs.getLanguage(language)) {
    // `ignoreIllegals` because a block that is still streaming is code cut off
    // mid-expression, and a grammar is entitled to refuse it.
    return hljs.highlight(code, { language, ignoreIllegals: true }).value;
  }

  return md.utils.escapeHtml(code);
}

/** An answer, or as much of one as has arrived, as HTML this window can show. */
export function render(markdown: string): string {
  return md.render(markdown);
}

/** What a copy button was standing for, when a click landed on one. */
export type Copyable = {
  /** The button itself, so that it can say the copy landed. */
  button: HTMLElement;
  /** The code it copies. */
  code: string;
};

/**
 * The code block a click asked to have copied, or `null` when the click was on
 * anything else.
 *
 * Answered here rather than walked in the component: the markup being walked is
 * the markup this module wrote, and a window should not have to know the shape
 * of it to find the button it just rendered.
 */
export function copyable(clicked: Element): Copyable | null {
  const button = clicked.closest(`[${COPY}]`);
  if (!(button instanceof HTMLElement)) return null;

  const code = button.closest(`.${BLOCK}`)?.querySelector("code");

  return code ? { button, code: code.textContent ?? "" } : null;
}

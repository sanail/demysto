import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";

// Each Demysto window is its own entry point rather than a route: the Palette
// has to appear instantly, so it must not load the code of windows it is not.
export default defineConfig({
  plugins: [svelte(), tailwindcss()],
  // Do not let Vite's screen clearing swallow Rust compiler output.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: { ignored: ["**/src-tauri/**", "**/crates/**"] },
  },
  build: {
    rollupOptions: {
      input: { main: "index.html" },
    },
  },
});

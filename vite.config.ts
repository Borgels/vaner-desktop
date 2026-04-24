import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";

// Tauri expects the Vite dev server on a fixed port so the Rust
// backend can point its WebView at it. See
// https://v2.tauri.app/start/frontend/sveltekit/
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [sveltekit()],
  clearScreen: false,
  server: {
    host: host || false,
    port: 1420,
    strictPort: true,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // Don't watch src-tauri — cargo watch handles that.
      ignored: ["**/src-tauri/**"],
    },
  },
});

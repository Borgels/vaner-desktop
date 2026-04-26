// Companion window route. SSR-off (Tauri WebView is a single client),
// prerender the static HTML so the SvelteKit static adapter emits a
// dedicated /companion/index.html that the Rust side can open as a
// second window via WebviewUrl::App("companion/...").
export const ssr = false;
export const prerender = true;

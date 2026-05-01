// Companion window route. SSR-off (Tauri WebView is a single client),
// prerender the static HTML so the SvelteKit static adapter emits a
// dedicated /companion/index.html that the Rust side can open as a
// second window via WebviewUrl::App("companion/...").
//
// trailingSlash='always' makes the adapter emit companion/index.html
// instead of companion.html. Without it Tauri loads companion.html,
// SvelteKit's router sees pathname '/companion.html' (which matches
// no route), and renders the framework's 404 page.
export const ssr = false;
export const prerender = true;
export const trailingSlash = "always";

// Onboarding window route. Static, prerendered, ssr-off — same shape as
// /companion. Tauri opens it via WebviewUrl::App("onboarding/"), which
// resolves to onboarding/index.html when trailingSlash='always' makes
// adapter-static emit the directory-style layout. Without this option
// the adapter emits a top-level onboarding.html, and SvelteKit's
// client-side router 404s on the literal pathname '/onboarding.html'.
export const ssr = false;
export const prerender = true;
export const trailingSlash = "always";

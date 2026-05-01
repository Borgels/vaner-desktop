// Windows subsystem opt-out so a Windows release build doesn't pop a
// console window. Ignored on Linux/macOS.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    apply_webkit_workarounds();
    vaner_desktop_lib::run();
}

/// Disable WebKitGTK's DMA-BUF / GBM compositing fast path on Linux.
///
/// On Ubuntu 24.04 with NVIDIA's proprietary driver, WebKitGTK 2.4x
/// requests a GBM buffer for hardware compositing and the kernel
/// returns `DRM_IOCTL_MODE_CREATE_DUMB: Permission denied`. The window
/// opens, the WebView claims to render, but the user sees a blank grey
/// rectangle. The same path also misbehaves on a number of mixed-GPU
/// laptops and on some Wayland sessions.
///
/// Disabling the DMA-BUF renderer plus the wider compositing-mode flag
/// forces a CPU paint that always works. Cost: a small amount of
/// scrolling overhead inside the popover, invisible at this UI's
/// modest size. We only set the variables when the user has not
/// already chosen a value, so power users on Intel / AMD can override
/// with `WEBKIT_DISABLE_DMABUF_RENDERER=0 vaner-desktop`.
#[cfg(target_os = "linux")]
fn apply_webkit_workarounds() {
    use std::env;
    if env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        unsafe {
            env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }
    if env::var_os("WEBKIT_DISABLE_COMPOSITING_MODE").is_none() {
        unsafe {
            env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn apply_webkit_workarounds() {
    // No equivalent issue on macOS / Windows WebKit.
}

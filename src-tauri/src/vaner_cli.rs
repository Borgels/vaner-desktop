//! Shared resolution of the `vaner` CLI binary.
//!
//! Prefers an explicit `VANER_BIN` env override (the AppImage and Windows
//! NSIS bundle can ship the CLI alongside) and otherwise falls back to a
//! `$PATH` lookup via the cross-platform [`which`] crate. On Windows that
//! handles `.exe`/`.cmd`/`.bat` extensions transparently, matching Python's
//! `shutil.which` semantics.

use std::path::PathBuf;

pub fn resolve_vaner_bin() -> Result<PathBuf, String> {
    if let Ok(explicit) = std::env::var("VANER_BIN") {
        if !explicit.is_empty() {
            return Ok(PathBuf::from(explicit));
        }
    }
    which::which("vaner").map_err(|_| {
        "Vaner binary not found on PATH. Install Vaner via vaner.ai/install or set $VANER_BIN."
            .to_string()
    })
}

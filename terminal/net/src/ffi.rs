//! Raw bindings against `terminal-net.c`.

use std::ffi::c_int;

unsafe extern "C" {
    /// Reads net.c's own `WINDOW_OFFSET` from a small function in
    /// `terminal-net.c` that mirrors its `#ifdef SMALL_SCREEN` exactly, so
    /// it can never drift from net.c's real value.
    pub(crate) fn window_offset() -> c_int;
}

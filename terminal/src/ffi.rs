//! Raw bindings against the mid-end.

use std::ffi::c_void;
use std::ptr;
use std::ptr::NonNull;

#[repr(C)]
struct RawMidend {
    _private: [u8; 0],
}

#[repr(C)]
pub(crate) struct RawGame {
    _private: [u8; 0],
}

#[repr(C)]
pub(crate) struct RawGameState {
    _private: [u8; 0],
}

#[repr(C)]
pub(crate) struct RawDrawingApi {
    _private: [u8; 0],
}

/// Mirrors `struct drawing` (`api`/`handle`, `puzzles.h`), so a callback
/// reached through a `drawing_api` function pointer can recover its own
/// `handle`.
#[repr(C)]
pub(crate) struct RawDrawing {
    pub(crate) api: *const RawDrawingApi,
    pub(crate) handle: *mut c_void,
}

extern "C" {
    fn midend_new(
        fe: *mut c_void,
        ourgame: *const RawGame,
        drapi: *const RawDrawingApi,
        drhandle: *mut c_void,
    ) -> *mut RawMidend;
    fn midend_new_game(me: *mut RawMidend);
    fn midend_redraw(me: *mut RawMidend);
    fn midend_free(me: *mut RawMidend);
}

/// A live mid-end handle. Every drawing call silently no-ops, except
/// `emit_state` (see `net.rs`), the one function our own `drawing_api`
/// actually implements.
pub(crate) struct Midend {
    raw: NonNull<RawMidend>,
}

impl Midend {
    /// `game` is the target puzzle's own `thegame` (e.g. `net.rs`'s).
    /// `drapi`/`drhandle` are passed straight through to `midend_new`.
    pub(crate) fn new(
        game: *const RawGame,
        drapi: *const RawDrawingApi,
        drhandle: *mut c_void,
    ) -> Midend {
        let raw = unsafe { midend_new(ptr::null_mut(), game, drapi, drhandle) };
        let raw = NonNull::new(raw).expect("midend_new returned null");
        Midend { raw }
    }

    /// Generates a fresh puzzle at whatever the current params are
    /// (`default_params()` unless changed).
    pub(crate) fn new_game(&self) {
        unsafe { midend_new_game(self.raw.as_ptr()) };
    }

    /// Triggers the backend's own redraw, which is where `emit_state`
    /// gets called.
    pub(crate) fn redraw(&self) {
        unsafe { midend_redraw(self.raw.as_ptr()) };
    }
}

impl Drop for Midend {
    fn drop(&mut self) {
        unsafe { midend_free(self.raw.as_ptr()) };
    }
}

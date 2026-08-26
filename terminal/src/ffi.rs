//! Raw bindings against the mid-end.

use std::ffi::{c_float, c_int, c_void};
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
    fn midend_process_key(me: *mut RawMidend, x: c_int, y: c_int, button: c_int) -> c_int;
    fn midend_timer(me: *mut RawMidend, tplus: c_float);
}

/// Net's rotation animation is `ROTATE_TIME` (net.c, 0.13 seconds); this
/// is comfortably longer, so one `midend_timer` call always finishes it.
const SKIP_ANIMATION_TIME: c_float = 1.0;

/// `midend_process_key`'s return value when it signals the front end
/// should quit (puzzles.h's `PKR_QUIT`).
const PKR_QUIT: c_int = 0;

/// A live mid-end handle. Every drawing call silently no-ops, except
/// `emit_state` and `status_bar` (see `net.rs`), which hand the puzzle
/// state and status text back to Rust.
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
    /// and `status_bar` get called.
    pub(crate) fn redraw(&self) {
        unsafe { midend_redraw(self.raw.as_ptr()) };
    }

    /// Sends one key/button press, then force-finishes any resulting
    /// animation so the next redraw shows the real final state, rather
    /// than the pre-move state an in-progress animation shows. Returns
    /// `false` if the mid-end signalled `PKR_QUIT`.
    pub(crate) fn process_key(&self, button: c_int) -> bool {
        let result = unsafe { midend_process_key(self.raw.as_ptr(), 0, 0, button) };
        unsafe { midend_timer(self.raw.as_ptr(), SKIP_ANIMATION_TIME) };
        result != PKR_QUIT
    }
}

impl Drop for Midend {
    fn drop(&mut self) {
        unsafe { midend_free(self.raw.as_ptr()) };
    }
}

extern "C" {
    fn smalloc(size: usize) -> *mut c_void;
}

/// Seeds the mid-end's RNG.
#[no_mangle]
extern "C" fn get_random_seed(randseed: *mut *mut c_void, randseedsize: *mut c_int) {
    let seed: u64 = rand::random();
    unsafe {
        let buf = smalloc(std::mem::size_of::<u64>()) as *mut u64;
        buf.write(seed);
        *randseed = buf as *mut c_void;
        *randseedsize = std::mem::size_of::<u64>() as c_int;
    }
}

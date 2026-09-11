//! Raw bindings against the mid-end.

use std::ffi::{c_double, c_float, c_int, c_void};
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
    api: *const RawDrawingApi,
    pub(crate) handle: *mut c_void,
}

unsafe extern "C" {
    fn midend_new(
        fe: *mut c_void,
        ourgame: *const RawGame,
        drapi: *const RawDrawingApi,
        drhandle: *mut c_void,
    ) -> *mut RawMidend;
    fn midend_new_game(me: *mut RawMidend);
    fn midend_restart_game(me: *mut RawMidend);
    fn midend_can_undo(me: *mut RawMidend) -> bool;
    fn midend_can_redo(me: *mut RawMidend) -> bool;
    fn midend_redraw(me: *mut RawMidend);
    fn midend_free(me: *mut RawMidend);
    fn midend_process_key(me: *mut RawMidend, x: c_int, y: c_int, button: c_int) -> c_int;
    fn midend_timer(me: *mut RawMidend, tplus: c_float);
    fn midend_size(
        me: *mut RawMidend,
        x: *mut c_int,
        y: *mut c_int,
        user_size: bool,
        device_pixel_ratio: c_double,
    );
    fn midend_tilesize(me: *mut RawMidend) -> c_int;
    /// Reads net.c's own `WINDOW_OFFSET` from a small function in
    /// `terminal.c` that mirrors its `#ifdef SMALL_SCREEN` exactly, so
    /// it can never drift from net.c's real value.
    pub(crate) fn window_offset() -> c_int;
}

/// Net's rotation animation is `ROTATE_TIME` (net.c, 0.13 seconds). This
/// value is comfortably longer, so one `midend_timer` call always
/// finishes it.
const SKIP_ANIMATION_TIME: c_float = 1.0;

/// The value `midend_process_key` returns when it signals the front end
/// should quit (puzzles.h's `PKR_QUIT`).
const PKR_QUIT: c_int = 0;

/// A live mid-end handle. Every drawing call silently no-ops, except
/// `emit_state` and `status_bar` (see `net.rs`), which hand the puzzle
/// state and status text back to Rust.
pub(crate) struct Midend {
    raw: NonNull<RawMidend>,
}

impl Midend {
    /// Takes the target puzzle's own `thegame` as `game` (e.g. `net.rs`'s),
    /// and passes `drapi` and `drhandle` straight through to `midend_new`.
    pub(crate) fn new(
        game: *const RawGame,
        drapi: *const RawDrawingApi,
        drhandle: *mut c_void,
    ) -> Midend {
        let raw = unsafe { midend_new(ptr::null_mut(), game, drapi, drhandle) };
        let raw = NonNull::new(raw).expect("midend_new returned null");

        Midend { raw }
    }

    /// Settles the mid-end on the backend's own preferred tile size
    /// (net.c's `PREFERRED_TILE_SIZE`), so `tilesize()` returns a real,
    /// known value instead of `0`. Passing `c_int::MAX` guarantees the
    /// preferred size always fits, since `midend_size` shrinks its result
    /// to fit the given space unless that space is already big enough not
    /// to matter. Must run after `new_game`. It writes into the
    /// mid-end's drawstate, which doesn't exist until a game does.
    fn fix_tilesize(&self) {
        let mut x = c_int::MAX;
        let mut y = c_int::MAX;

        unsafe { midend_size(self.raw.as_ptr(), &mut x, &mut y, false, 1.0) };
    }

    /// The tile size settled on by `fix_tilesize`, needed to convert a
    /// tile position into the pixel coordinates `process_click` expects.
    pub(crate) fn tilesize(&self) -> c_int {
        unsafe { midend_tilesize(self.raw.as_ptr()) }
    }

    /// Generates a fresh puzzle at whatever the current params are
    /// (`default_params()` unless changed).
    pub(crate) fn new_game(&self) {
        unsafe { midend_new_game(self.raw.as_ptr()) };
        self.fix_tilesize();
    }

    /// Returns the current puzzle to its starting position, keeping the
    /// move history so the restart itself can be undone.
    pub(crate) fn restart_game(&self) {
        unsafe { midend_restart_game(self.raw.as_ptr()) };
    }

    pub(crate) fn can_undo(&self) -> bool {
        unsafe { midend_can_undo(self.raw.as_ptr()) }
    }

    pub(crate) fn can_redo(&self) -> bool {
        unsafe { midend_can_redo(self.raw.as_ptr()) }
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

    /// Sends one mouse button press at the given pixel coordinates,
    /// then force-finishes any resulting animation the same way
    /// `process_key` does. Returns `false` if the mid-end signalled
    /// `PKR_QUIT`.
    pub(crate) fn process_click(&self, x: c_int, y: c_int, button: c_int) -> bool {
        let result = unsafe { midend_process_key(self.raw.as_ptr(), x, y, button) };
        unsafe { midend_timer(self.raw.as_ptr(), SKIP_ANIMATION_TIME) };

        result != PKR_QUIT
    }
}

impl Drop for Midend {
    fn drop(&mut self) {
        unsafe { midend_free(self.raw.as_ptr()) };
    }
}

unsafe extern "C" {
    fn smalloc(size: usize) -> *mut c_void;
}

/// Seeds the mid-end's RNG.
#[unsafe(no_mangle)]
extern "C" fn get_random_seed(randseed: *mut *mut c_void, randseedsize: *mut c_int) {
    let seed: u64 = rand::random();

    unsafe {
        let buf = smalloc(std::mem::size_of::<u64>()) as *mut u64;
        buf.write(seed);
        *randseed = buf as *mut c_void;
        *randseedsize = std::mem::size_of::<u64>() as c_int;
    }
}

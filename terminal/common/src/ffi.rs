//! Raw bindings against the mid-end.

use std::ffi::{CStr, c_char, c_double, c_float, c_int, c_void};
use std::ptr;
use std::ptr::NonNull;

#[repr(C)]
struct RawMidend {
    _private: [u8; 0],
}

#[repr(C)]
pub struct RawGame {
    _private: [u8; 0],
}

#[repr(C)]
pub struct RawDrawingApi {
    _private: [u8; 0],
}

/// Mirrors `struct drawing` (`api`/`handle`, `puzzles.h`), so a callback
/// reached through a `drawing_api` function pointer can recover its own
/// `handle`.
#[repr(C)]
pub struct RawDrawing {
    api: *const RawDrawingApi,
    pub handle: *mut c_void,
}

#[repr(C)]
struct RawGameParams {
    _private: [u8; 0],
}

/// Mirrors `struct preset_menu_entry` (`puzzles.h`). An entry holds a
/// preset when `params` is set and a submenu when `submenu` is, never
/// both.
#[repr(C)]
struct RawPresetMenuEntry {
    title: *const c_char,
    params: *mut RawGameParams,
    submenu: *const RawPresetMenu,
    id: c_int,
}

/// Mirrors `struct preset_menu` (`puzzles.h`).
#[repr(C)]
struct RawPresetMenu {
    n_entries: c_int,
    entries_size: c_int,
    entries: *const RawPresetMenuEntry,
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
    fn midend_get_presets(me: *mut RawMidend, id_limit: *mut c_int) -> *const RawPresetMenu;
    fn midend_which_preset(me: *mut RawMidend) -> c_int;
    fn midend_set_params(me: *mut RawMidend, params: *mut RawGameParams);
    fn preset_menu_lookup_by_id(menu: *const RawPresetMenu, id: c_int) -> *mut RawGameParams;
}

/// Longer than a common animation length, so one `midend_timer` call
/// finishes the animation.
const SKIP_ANIMATION_TIME: c_float = 1.0;

/// The value `midend_process_key` returns when it signals the front end
/// should quit (puzzles.h's `PKR_QUIT`).
const PKR_QUIT: c_int = 0;

/// The value `midend_which_preset` returns when the current parameters
/// match no preset, which is what a custom game gives.
const NO_PRESET: c_int = -1;

/// The mid-end generates games, keeps the undo history and handles
/// timers and game IDs. The drawing calls it makes silently no-op,
/// since each game hands its state to Rust through its own hooks.
pub struct Midend {
    raw: NonNull<RawMidend>,
}

/// One preset the mid-end offers, under the id it allocated for it.
/// The id is what `which_preset` returns and what `set_preset` takes.
pub struct Preset {
    pub id: usize,
    pub title: String,
}

impl Midend {
    /// Takes the target puzzle's own `thegame` as `game` and passes
    /// `drapi` and `drhandle` straight through to `midend_new`.
    ///
    /// # Safety
    ///
    /// All three pointers must stay valid for as long as the mid-end
    /// lives, since it keeps them and uses them on later calls.
    pub unsafe fn new(
        game: *const RawGame,
        drapi: *const RawDrawingApi,
        drhandle: *mut c_void,
    ) -> Midend {
        let raw = unsafe { midend_new(ptr::null_mut(), game, drapi, drhandle) };
        let raw = NonNull::new(raw).expect("midend_new returned null");

        Midend { raw }
    }

    /// Settles the mid-end on the backend's own preferred tile size
    /// (its `PREFERRED_TILE_SIZE`), so `tilesize()` returns a real,
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
    pub fn tilesize(&self) -> c_int {
        unsafe { midend_tilesize(self.raw.as_ptr()) }
    }

    /// Generates a fresh puzzle at whatever the current params are
    /// (`default_params()` unless changed).
    pub fn new_game(&self) {
        unsafe { midend_new_game(self.raw.as_ptr()) };
        self.fix_tilesize();
    }

    /// Returns the current puzzle to its starting position, keeping the
    /// move history so the restart itself can be undone.
    pub fn restart_game(&self) {
        unsafe { midend_restart_game(self.raw.as_ptr()) };
    }

    pub fn can_undo(&self) -> bool {
        unsafe { midend_can_undo(self.raw.as_ptr()) }
    }

    pub fn can_redo(&self) -> bool {
        unsafe { midend_can_redo(self.raw.as_ptr()) }
    }

    /// Triggers the backend's own redraw, which is where `emit_state`
    /// and `status_bar` get called.
    pub fn redraw(&self) {
        unsafe { midend_redraw(self.raw.as_ptr()) };
    }

    /// Every preset the mid-end offers, flattened out of the menu tree
    /// it hands back. The tree belongs to the mid-end and outlives this
    /// call, so the titles are copied out of it.
    pub fn presets(&self) -> Vec<Preset> {
        let menu = unsafe { midend_get_presets(self.raw.as_ptr(), ptr::null_mut()) };
        let mut presets = Vec::new();

        collect_presets(menu, &mut presets);
        presets
    }

    /// The preset the current game matches, or `None` for a custom one.
    pub fn which_preset(&self) -> Option<usize> {
        let id = unsafe { midend_which_preset(self.raw.as_ptr()) };

        (id != NO_PRESET).then_some(id as usize)
    }

    /// Switches to a preset and starts a fresh game at it, the way the
    /// other front ends do.
    pub fn set_preset(&self, id: usize) {
        let menu = unsafe { midend_get_presets(self.raw.as_ptr(), ptr::null_mut()) };
        let params = unsafe { preset_menu_lookup_by_id(menu, id as c_int) };
        assert!(!params.is_null(), "no preset has id {id}");

        unsafe { midend_set_params(self.raw.as_ptr(), params) };
        self.new_game();
    }

    /// Sends one key/button press, then force-finishes any resulting
    /// animation so the next redraw shows the real final state, rather
    /// than the pre-move state an in-progress animation shows. Returns
    /// `false` if the mid-end signalled `PKR_QUIT`.
    pub fn process_key(&self, button: c_int) -> bool {
        let result = unsafe { midend_process_key(self.raw.as_ptr(), 0, 0, button) };
        unsafe { midend_timer(self.raw.as_ptr(), SKIP_ANIMATION_TIME) };

        result != PKR_QUIT
    }

    /// Sends one mouse button press at the given pixel coordinates,
    /// then force-finishes any resulting animation the same way
    /// `process_key` does. Returns `false` if the mid-end signalled
    /// `PKR_QUIT`.
    pub fn process_click(&self, x: c_int, y: c_int, button: c_int) -> bool {
        let result = unsafe { midend_process_key(self.raw.as_ptr(), x, y, button) };
        unsafe { midend_timer(self.raw.as_ptr(), SKIP_ANIMATION_TIME) };

        result != PKR_QUIT
    }
}

/// Walks one level of the preset menu, following submenus down.
fn collect_presets(menu: *const RawPresetMenu, presets: &mut Vec<Preset>) {
    let menu = unsafe { &*menu };
    let entries = unsafe { std::slice::from_raw_parts(menu.entries, menu.n_entries as usize) };

    for entry in entries {
        if entry.params.is_null() {
            collect_presets(entry.submenu, presets);
            continue;
        }

        let title = unsafe { CStr::from_ptr(entry.title) }.to_string_lossy().into_owned();
        presets.push(Preset { id: entry.id as usize, title });
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

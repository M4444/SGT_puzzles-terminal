//! A live, playable game: the mid-end and the puzzle its callbacks
//! hand back.

use crate::ffi::{self, Midend, Preset};
use std::ffi::{c_int, c_void};

/// What `dr->handle` points at, recovered by every drawing-API
/// callback. `P` is the game's own puzzle type, which its callbacks
/// build.
pub struct DrawHandle<P> {
    pub puzzle: Option<P>,
}

impl<P> Default for DrawHandle<P> {
    fn default() -> DrawHandle<P> {
        DrawHandle { puzzle: None }
    }
}

/// A session owns the mid-end and the drawing handle, for as long as
/// the game is played.
pub struct Session<P> {
    midend: Midend,
    handle: Box<DrawHandle<P>>,
}

impl<P> Session<P> {
    /// Starts a game of the puzzle this binary is built with.
    pub fn start() -> Session<P> {
        let mut handle = Box::new(DrawHandle::default());
        let handle_ptr = &mut *handle as *mut DrawHandle<P> as *mut c_void;

        let midend = unsafe { Midend::new(&ffi::THEGAME, &ffi::TERMINAL_DRAWING_API, handle_ptr) };
        midend.new_game();
        midend.redraw();

        Session { midend, handle }
    }

    /// The puzzle state as of the most recent `start()`/`process_key()`.
    pub fn puzzle(&self) -> &P {
        self.handle.puzzle.as_ref().expect("emit_state was not called")
    }

    /// Sends one raw key/button code straight to the mid-end. Returns
    /// `false` if it signalled quit.
    pub fn process_key(&mut self, button: c_int) -> bool {
        self.midend.process_key(button)
    }

    /// Sends one mouse button press at a pixel position in the game's
    /// own layout. Returns `false` if it signalled quit.
    pub fn process_click(&mut self, x: c_int, y: c_int, button: c_int) -> bool {
        self.midend.process_click(x, y, button)
    }

    /// Returns the puzzle to its starting position. Unlike the other
    /// actions this has no keystroke of its own, so it goes straight
    /// to the mid-end and needs its own redraw.
    pub fn restart(&mut self) {
        self.midend.restart_game();
        self.midend.redraw();
    }

    pub fn can_undo(&self) -> bool {
        self.midend.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.midend.can_redo()
    }

    pub fn can_solve(&self) -> bool {
        unsafe { ffi::can_solve() }
    }

    pub fn presets(&self) -> Vec<Preset> {
        self.midend.presets()
    }

    pub fn which_preset(&self) -> Option<usize> {
        self.midend.which_preset()
    }

    /// Starts a fresh game at the given preset, which needs its own
    /// redraw the way `restart` does.
    pub fn set_preset(&mut self, id: usize) {
        self.midend.set_preset(id);
        self.midend.redraw();
    }

    pub fn wants_status_bar(&self) -> bool {
        self.midend.wants_status_bar()
    }

    /// The tile size the mid-end settled on, which a game needs to turn
    /// a tile into the pixel coordinates `process_click` expects.
    pub fn tilesize(&self) -> c_int {
        self.midend.tilesize()
    }
}

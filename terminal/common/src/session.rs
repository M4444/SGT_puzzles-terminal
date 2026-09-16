//! A live, playable game: the mid-end and the puzzle its callbacks
//! hand back.

use crate::ffi::{Midend, Preset, RawDrawingApi, RawGame};
use std::ffi::{c_int, c_void};

/// The front end's state, recovered from `dr->handle` by every
/// drawing-API callback. `P` is the game's own puzzle type, which its
/// callbacks build.
pub struct Frontend<P> {
    pub puzzle: Option<P>,
}

impl<P> Default for Frontend<P> {
    fn default() -> Frontend<P> {
        Frontend { puzzle: None }
    }
}

/// A session owns the mid-end and the front end state, for as long as
/// the game is played.
pub struct Session<P> {
    midend: Midend,
    frontend: Box<Frontend<P>>,
}

impl<P> Session<P> {
    /// Starts a game of the puzzle `game` describes, with `drapi` as
    /// the mid-end's drawing API.
    ///
    /// # Safety
    ///
    /// Both pointers must stay valid for as long as the session lives,
    /// since the mid-end keeps them.
    pub unsafe fn new(game: *const RawGame, drapi: *const RawDrawingApi) -> Session<P> {
        let mut frontend = Box::new(Frontend::default());
        let frontend_ptr = &mut *frontend as *mut Frontend<P> as *mut c_void;

        let midend = unsafe { Midend::new(game, drapi, frontend_ptr) };
        midend.new_game();
        midend.redraw();

        Session { midend, frontend }
    }

    /// The puzzle state as of the most recent `new()`/`process_key()`.
    pub fn puzzle(&self) -> &P {
        self.frontend.puzzle.as_ref().expect("emit_state was not called")
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

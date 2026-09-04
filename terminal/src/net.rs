//! Generating and reading Net puzzles.

use crate::ffi::{window_offset, Midend, RawDrawing, RawDrawingApi, RawGame, RawGameState};
use crate::menu::{LegendEntry, LegendGroup};
use std::ffi::{c_char, c_int, c_void, CStr};

extern "C" {
    #[link_name = "thegame"]
    static THEGAME: RawGame;
    #[link_name = "terminal_drawing_api"]
    static TERMINAL_DRAWING_API: RawDrawingApi;
}

/// Which directions a tile has a wire pointing in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Wires {
    pub right: bool,
    pub up: bool,
    pub left: bool,
    pub down: bool,
}

/// Bit flags within a direction nibble, matching net.c's `R`/`U`/`L`/`D`
/// `#define`s.
const RIGHT: u8 = 0x1;
const UP: u8 = 0x2;
const LEFT: u8 = 0x4;
const DOWN: u8 = 0x8;

/// Bit flag within a tile's byte marking it locked, matching net.c's
/// `LOCKED` `#define`.
const LOCKED_BIT: u8 = 0x10;

impl Wires {
    fn from_bits(bits: u8) -> Wires {
        Wires {
            right: bits & RIGHT != 0,
            up: bits & UP != 0,
            left: bits & LEFT != 0,
            down: bits & DOWN != 0,
        }
    }
}

/// Which of a tile's four edges carry a barrier, a wall no wire can
/// cross. Both tiles either side of a wall record it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Barriers {
    pub right: bool,
    pub up: bool,
    pub left: bool,
    pub down: bool,
}

impl Barriers {
    fn from_bits(bits: u8) -> Barriers {
        Barriers {
            right: bits & RIGHT != 0,
            up: bits & UP != 0,
            left: bits & LEFT != 0,
            down: bits & DOWN != 0,
        }
    }
}

pub type GridDimensions = (usize, usize);
pub type TileCoord = (usize, usize);
pub type Tiles = Vec<Vec<Tile>>;

/// A tile coordinate's neighbours one step over in each direction.
pub trait TileCoordNeighbors {
    fn right(&self) -> TileCoord;
    fn top(&self) -> TileCoord;
    fn left(&self) -> TileCoord;
    fn bottom(&self) -> TileCoord;
}

impl TileCoordNeighbors for TileCoord {
    fn right(&self) -> TileCoord {
        let (x, y) = *self;
        (x + 1, y)
    }
    fn top(&self) -> TileCoord {
        let (x, y) = *self;
        (x, y - 1)
    }
    fn left(&self) -> TileCoord {
        let (x, y) = *self;
        (x - 1, y)
    }
    fn bottom(&self) -> TileCoord {
        let (x, y) = *self;
        (x, y + 1)
    }
}

/// A single tile's wires and barriers, and whether it's currently
/// powered or locked.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Tile {
    pub wires: Wires,
    pub barriers: Barriers,
    pub powered: bool,
    pub locked: bool,
}

/// The keyboard cursor's position and whether it's currently shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cursor {
    pub position: TileCoord,
    pub visible: bool,
}

/// A Net puzzle, indexed `[row][column]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetPuzzle {
    pub dimensions: GridDimensions,
    pub tiles: Tiles,
    pub cursor: Cursor,
    pub source: TileCoord,
    pub origin: TileCoord,
    pub status: String,
}

/// The front end's state, recovered from `dr->handle` by every
/// drawing-API callback.
#[derive(Default)]
struct Frontend {
    puzzle: Option<NetPuzzle>,
}

/// Called from net.c's `game_redraw`, through our own `drawing_api`
/// (`terminal_drawing_api`), with the same `state`/`active` that
/// `compute_active` just produced.
#[no_mangle]
extern "C" fn rust_emit_state(
    dr: *mut RawDrawing,
    _state: *const RawGameState,
    active: *const u8,
    tiles: *const u8,
    barriers: *const u8,
    width: c_int,
    height: c_int,
    cur_x: c_int,
    cur_y: c_int,
    cur_visible: bool,
    source_x: c_int,
    source_y: c_int,
    org_x: c_int,
    org_y: c_int,
) {
    let width = width as usize;
    let height = height as usize;
    let frontend = unsafe { &mut *((*dr).handle as *mut Frontend) };

    let raw_active = unsafe { std::slice::from_raw_parts(active, width * height) };
    let raw_tiles = unsafe { std::slice::from_raw_parts(tiles, width * height) };
    let raw_barriers = unsafe { std::slice::from_raw_parts(barriers, width * height) };
    let tiles: Vec<Tile> = raw_tiles
        .iter()
        .zip(raw_active.iter())
        .zip(raw_barriers.iter())
        .map(|((&bits, &active), &barriers)| Tile {
            wires: Wires::from_bits(bits),
            barriers: Barriers::from_bits(barriers),
            powered: active != 0,
            locked: bits & LOCKED_BIT != 0,
        })
        .collect();

    frontend.puzzle = Some(NetPuzzle {
        dimensions: (width, height),
        tiles: tiles.chunks(width).map(|row| row.to_vec()).collect(),
        cursor: Cursor { position: (cur_x as usize, cur_y as usize), visible: cur_visible },
        source: (source_x as usize, source_y as usize),
        origin: (org_x as usize, org_y as usize),
        status: String::new(),
    });
}

/// Called from net.c's `status_bar`, right after `rust_emit_state` within
/// the same `game_redraw`.
#[no_mangle]
extern "C" fn rust_status_bar(dr: *mut RawDrawing, text: *const c_char) {
    let frontend = unsafe { &mut *((*dr).handle as *mut Frontend) };
    let text = unsafe { CStr::from_ptr(text) }.to_string_lossy().into_owned();
    frontend.puzzle.as_mut().expect("emit_state was not called").status = text;
}

/// Generates a fresh Net puzzle via FFI and reads it from the live state.
pub fn generate() -> NetPuzzle {
    let mut frontend = Box::new(Frontend::default());
    let frontend_ptr = &mut *frontend as *mut Frontend as *mut c_void;

    let midend = Midend::new(unsafe { &THEGAME }, unsafe { &TERMINAL_DRAWING_API }, frontend_ptr);
    midend.new_game();
    midend.redraw();

    frontend.puzzle.take().expect("emit_state was not called")
}

/// Net's mouse inputs.
const MOUSE_CONTROLS: &[LegendEntry] = &[
    LegendEntry { input: "Left button", description: "rotate tile 90° counter clockwise" },
    LegendEntry { input: "Right button", description: "rotate tile 90° clockwise" },
    LegendEntry { input: "Middle button", description: "lock / unlock tile" },
];

/// Net's keyboard inputs.
const KEYBOARD_CONTROLS: &[LegendEntry] = &[
    LegendEntry { input: "Arrows", description: "move cursor" },
    LegendEntry { input: "Ctrl + arrows", description: "move source tile" },
    LegendEntry { input: "Shift + arrows", description: "move origin (wrapping)" },
    LegendEntry { input: "Ctrl + Shift + arrows", description: "move source and origin" },
    LegendEntry { input: "A / Enter", description: "rotate tile 90° counter clockwise" },
    LegendEntry { input: "D", description: "rotate tile 90° clockwise" },
    LegendEntry { input: "F", description: "rotate tile 180°" },
    LegendEntry { input: "S / Space", description: "lock / unlock tile" },
    LegendEntry { input: "J", description: "jumble unlocked tiles" },
];

pub(crate) const GAME_CONTROLS: &[LegendGroup] = &[
    LegendGroup { label: Some("Mouse:"), entries: MOUSE_CONTROLS },
    LegendGroup { label: Some("Keyboard:"), entries: KEYBOARD_CONTROLS },
];

/// A live, playable Net session: owns the mid-end and the front end
/// state, for as long as the session is played.
pub struct Session {
    midend: Midend,
    frontend: Box<Frontend>,
}

impl Session {
    pub fn new() -> Session {
        let mut frontend = Box::new(Frontend::default());
        let frontend_ptr = &mut *frontend as *mut Frontend as *mut c_void;

        let midend =
            Midend::new(unsafe { &THEGAME }, unsafe { &TERMINAL_DRAWING_API }, frontend_ptr);
        midend.new_game();
        midend.redraw();

        Session { midend, frontend }
    }

    /// The puzzle state as of the most recent `new()`/`process_key()`.
    pub fn puzzle(&self) -> &NetPuzzle {
        self.frontend.puzzle.as_ref().expect("emit_state was not called")
    }

    /// Sends one raw key/button code straight to the mid-end. Returns
    /// `false` if it signalled quit.
    pub fn process_key(&mut self, button: c_int) -> bool {
        self.midend.process_key(button)
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

    /// Sends one mouse button press on the given tile, converting it
    /// into the pixel coordinates net.c's own click handling expects.
    /// Returns `false` if it signalled quit.
    pub fn process_click(&mut self, (tile_x, tile_y): TileCoord, button: c_int) -> bool {
        let tilesize = self.midend.tilesize();
        let line_thick = line_thick(tilesize);
        let window_offset = unsafe { window_offset() };
        // Centers the click in the tile.
        let x = window_offset + line_thick + tile_x as c_int * tilesize + tilesize / 2;
        let y = window_offset + line_thick + tile_y as c_int * tilesize + tilesize / 2;
        self.midend.process_click(x, y, button)
    }

    pub fn exclude_locked(&self, tile_coords: &[TileCoord]) -> Vec<TileCoord> {
        tile_coords
            .iter()
            .copied()
            .filter(|&(x, y)| !self.puzzle().tiles[y][x].locked)
            .collect()
    }
}

/// Computes net.c's `LINE_THICK`, always derived from the tile size
/// the same way regardless of platform.
fn line_thick(tilesize: c_int) -> c_int {
    (tilesize + 47) / 48
}

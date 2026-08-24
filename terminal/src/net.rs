//! Generating and reading Net puzzles.

use crate::ffi::{Midend, RawDrawing, RawDrawingApi, RawGame, RawGameState};
use std::ffi::{c_int, c_void};

extern "C" {
    #[link_name = "thegame"]
    static THEGAME: RawGame;
    #[link_name = "terminal_drawing_api"]
    static TERMINAL_DRAWING_API: RawDrawingApi;
}

/// Which directions a tile has a wire pointing in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Wires {
    pub right: bool,
    pub up: bool,
    pub left: bool,
    pub down: bool,
}

/// Bit flags within a tile's wire nibble, matching net.c's `R`/`U`/`L`/`D`
/// `#define`s.
const WIRE_RIGHT: u8 = 0x1;
const WIRE_UP: u8 = 0x2;
const WIRE_LEFT: u8 = 0x4;
const WIRE_DOWN: u8 = 0x8;

impl Wires {
    fn from_bits(bits: u8) -> Wires {
        Wires {
            right: bits & WIRE_RIGHT != 0,
            up: bits & WIRE_UP != 0,
            left: bits & WIRE_LEFT != 0,
            down: bits & WIRE_DOWN != 0,
        }
    }
}

pub type GridDimensions = (usize, usize);
pub type TileCoord = (usize, usize);
pub type Tiles = Vec<Vec<Tile>>;

/// A single tile's wires and whether it's currently powered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tile {
    pub wires: Wires,
    pub powered: bool,
}

/// The keyboard cursor's position and whether it's currently shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cursor {
    pub position: TileCoord,
    pub visible: bool,
}

/// A Net puzzle, indexed `[row][column]`.
#[derive(Debug, PartialEq, Eq)]
pub struct NetPuzzle {
    pub dimensions: GridDimensions,
    pub wrapping: bool,
    pub tiles: Tiles,
    pub cursor: Cursor,
}

/// Where `emit_state` (called during `Midend::redraw`) deposits the
/// puzzle it receives, via the `drhandle` passed to `Midend::new`.
#[derive(Default)]
struct EmitContext {
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
    wrapping: bool,
    width: c_int,
    height: c_int,
    cur_x: c_int,
    cur_y: c_int,
    cur_visible: bool,
) {
    let width = width as usize;
    let height = height as usize;
    let context = unsafe { &mut *((*dr).handle as *mut EmitContext) };

    let raw_active = unsafe { std::slice::from_raw_parts(active, width * height) };
    let raw_tiles = unsafe { std::slice::from_raw_parts(tiles, width * height) };
    let tiles: Vec<Tile> = raw_tiles
        .iter()
        .zip(raw_active.iter())
        .map(|(&bits, &active)| Tile {
            wires: Wires::from_bits(bits),
            powered: active != 0,
        })
        .collect();

    context.puzzle = Some(NetPuzzle {
        dimensions: (width, height),
        wrapping,
        tiles: tiles.chunks(width).map(|row| row.to_vec()).collect(),
        cursor: Cursor {
            position: (cur_x as usize, cur_y as usize),
            visible: cur_visible,
        },
    });
}

/// Generates a fresh Net puzzle via FFI and reads it from the live state.
pub fn generate() -> NetPuzzle {
    let mut context = Box::new(EmitContext::default());
    let context_ptr = &mut *context as *mut EmitContext as *mut c_void;

    let midend = Midend::new(unsafe { &THEGAME }, unsafe { &TERMINAL_DRAWING_API }, context_ptr);
    midend.new_game();
    midend.redraw();

    context.puzzle.take().expect("emit_state was not called")
}

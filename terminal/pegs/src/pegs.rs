//! Generating and reading Pegs puzzles, with the menu tabs Pegs offers.

use common::board::{Grid, TileCoord};
use common::ffi::{Preset, RawDrawing};
use common::menu::{self, LegendEntry, LegendGroup, TabSpec};
use common::session::DrawHandle;
use std::ffi::{c_char, c_int};

/// What a tile on the grid holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tile {
    Hole,
    Peg,
    Obstacle,
}

pub(crate) type Tiles = Grid<Tile>;

/// A grid value's meaning, matching pegs.c's `GRID_HOLE` and `GRID_PEG`
/// `#define`s. Anything else is a position off the board.
const HOLE: u8 = 0;
const PEG: u8 = 1;

impl Tile {
    fn from_value(value: u8) -> Tile {
        match value {
            HOLE => Tile::Hole,
            PEG => Tile::Peg,
            _ => Tile::Obstacle,
        }
    }
}

/// The keyboard cursor's position, whether it's shown and whether the
/// peg on it has been picked up to jump.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Cursor {
    pub position: TileCoord,
    pub visible: bool,
    pub jumping: bool,
}

/// A Pegs puzzle.
pub(crate) struct PegsPuzzle {
    pub tiles: Tiles,
    pub cursor: Cursor,
}

/// Mirrors `struct live_state` (pegs.c), what Pegs hands over through
/// the drawing API's `emit_state`.
#[repr(C)]
struct RawLiveState {
    grid: *const u8,
    width: c_int,
    height: c_int,
    cur_x: c_int,
    cur_y: c_int,
    cur_visible: bool,
    cur_jumping: bool,
}

/// Called from pegs.c's `game_redraw`, through the drawing API.
#[unsafe(no_mangle)]
extern "C" fn rust_emit_state(dr: *mut RawDrawing, data: *const RawLiveState) {
    let data = unsafe { &*data };
    let width = data.width as usize;
    let height = data.height as usize;
    let handle = unsafe { &mut *((*dr).handle as *mut DrawHandle<PegsPuzzle>) };

    let grid = unsafe { std::slice::from_raw_parts(data.grid, width * height) };
    let tiles: Vec<Tile> = grid.iter().map(|&value| Tile::from_value(value)).collect();

    handle.puzzle = Some(PegsPuzzle {
        tiles: tiles.chunks(width).map(|row| row.to_vec()).collect(),
        cursor: Cursor {
            position: (data.cur_x as usize, data.cur_y as usize),
            visible: data.cur_visible,
            jumping: data.cur_jumping,
        },
    });
}

/// Pegs has no status bar, so the mid-end never asks for one. The
/// shared `terminal.c` still calls this, so it has to exist.
#[unsafe(no_mangle)]
extern "C" fn rust_status_bar(_dr: *mut RawDrawing, _text: *const c_char) {}

/// The keyboard inputs Pegs takes.
const KEYBOARD_CONTROLS: &[LegendEntry] = &[
    LegendEntry { input: "Arrows", description: "move cursor / jump with picked-up peg" },
    LegendEntry { input: "Enter / Space", description: "pick up / put down peg" },
];

const GAME_CONTROLS: &[LegendGroup] =
    &[LegendGroup { label: Some("Keyboard:"), entries: KEYBOARD_CONTROLS }];

/// Pegs has no menu choices of its own.
#[derive(Clone, Copy)]
pub(crate) enum PegsAction {}

/// Pegs' menu tabs, just the usuals since it doesn't have ones that are
/// specific to it.
pub(crate) fn tab_specs(presets: &[Preset], can_solve: bool) -> Vec<TabSpec<PegsAction>> {
    menu::tabs(presets, can_solve, Vec::new(), GAME_CONTROLS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::session::Session;

    /// Pegs starts on the 7x7 cross, with every tile holding a peg
    /// except the hole in the middle.
    #[test]
    fn a_fresh_game_is_the_cross() {
        let session = Session::<PegsPuzzle>::start();
        let tiles = &session.puzzle().tiles;

        assert_eq!(tiles.len(), 7);
        assert_eq!(tiles[0].len(), 7);
        assert_eq!(tiles[3][3], Tile::Hole);
        assert_eq!(tiles[0][0], Tile::Obstacle);
        assert_eq!(tiles[3][0], Tile::Peg);
    }

    /// Pegs has no solver, so its menu leaves out Solve.
    #[test]
    fn pegs_cannot_solve() {
        assert!(!Session::<PegsPuzzle>::start().can_solve());
    }
}

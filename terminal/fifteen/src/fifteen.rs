//! Generating and reading Fifteen puzzles, with the menu tabs Fifteen
//! offers.

use common::board::{Grid, GridDimensions, Puzzle};
use common::ffi::{Preset, RawDrawing};
use common::menu::{self, LegendEntry, LegendGroup, TabSpec};
use common::session::DrawHandle;
use std::ffi::{CStr, c_char, c_int};

/// What a tile on the grid holds. The gap is the one square a
/// neighbouring tile can slide into.
#[derive(Clone, Copy)]
pub(crate) enum Tile {
    Number(u8),
    Gap,
}

pub(crate) type Tiles = Grid<Tile>;

/// The gap's own value, matching fifteen.c, whose tile array holds a
/// zero where the gap is.
const GAP: c_int = 0;

impl Tile {
    fn from_value(value: c_int) -> Tile {
        match value {
            GAP => Tile::Gap,
            number => Tile::Number(number as u8),
        }
    }
}

/// Fifteen has no menu choices of its own.
#[derive(Clone, Copy)]
pub(crate) enum FifteenAction {}

/// A Fifteen puzzle.
pub(crate) struct FifteenPuzzle {
    pub tiles: Tiles,
    pub status: String,
}

impl Puzzle for FifteenPuzzle {
    fn dimensions(&self) -> GridDimensions {
        (self.tiles[0].len(), self.tiles.len())
    }
}

/// Mirrors `struct live_state` (fifteen.c), what Fifteen hands over
/// through the drawing API's `emit_state`.
#[repr(C)]
struct RawLiveState {
    tiles: *const c_int,
    width: c_int,
    height: c_int,
}

/// Called from fifteen.c's `game_redraw`, through the drawing API.
#[unsafe(no_mangle)]
extern "C" fn rust_emit_state(dr: *mut RawDrawing, data: *const RawLiveState) {
    let data = unsafe { &*data };
    let width = data.width as usize;
    let height = data.height as usize;
    let handle = unsafe { &mut *((*dr).handle as *mut DrawHandle<FifteenPuzzle>) };

    let values = unsafe { std::slice::from_raw_parts(data.tiles, width * height) };
    let tiles: Vec<Tile> = values.iter().map(|&value| Tile::from_value(value)).collect();

    handle.puzzle = Some(FifteenPuzzle {
        tiles: tiles.chunks(width).map(|row| row.to_vec()).collect(),
        status: String::new(),
    });
}

/// Called from fifteen.c's `status_bar`, right after `rust_emit_state`
/// within the same `game_redraw`.
#[unsafe(no_mangle)]
extern "C" fn rust_status_bar(dr: *mut RawDrawing, text: *const c_char) {
    let handle = unsafe { &mut *((*dr).handle as *mut DrawHandle<FifteenPuzzle>) };
    let text = unsafe { CStr::from_ptr(text) }.to_string_lossy().into_owned();

    handle.puzzle.as_mut().expect("emit_state was not called").status = text;
}

/// The keyboard inputs Fifteen takes.
const KEYBOARD_CONTROLS: &[LegendEntry] = &[
    LegendEntry { input: "Arrows", description: "slide the tile that way into the gap" },
    LegendEntry { input: "H", description: "make a move for me" },
];

const GAME_CONTROLS: &[LegendGroup] =
    &[LegendGroup { label: Some("Keyboard:"), entries: KEYBOARD_CONTROLS }];

/// Fifteen's menu tabs, just the usuals since it doesn't have ones that
/// are specific to it.
pub(crate) fn tab_specs(presets: &[Preset], can_solve: bool) -> Vec<TabSpec<FifteenAction>> {
    menu::tabs(presets, can_solve, Vec::new(), GAME_CONTROLS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::session::Session;

    /// The one preset's board, with one gap among the fifteen numbers.
    #[test]
    fn starting_board_is_four_by_four_with_one_gap() {
        let session = Session::<FifteenPuzzle>::start();
        let tiles = &session.puzzle().tiles;
        let gaps = tiles
            .iter()
            .flatten()
            .filter(|tile| matches!(tile, Tile::Gap))
            .count();

        assert_eq!(tiles.len(), 4);
        assert_eq!(tiles[0].len(), 4);
        assert_eq!(gaps, 1);
    }

    /// Fifteen counts the moves made in its status bar.
    #[test]
    fn status_starts_with_no_moves() {
        let session = Session::<FifteenPuzzle>::start();

        assert_eq!(session.puzzle().status, "Moves: 0");
    }
}

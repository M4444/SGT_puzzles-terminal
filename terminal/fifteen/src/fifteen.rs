//! Generating and reading Fifteen puzzles, with the menu tabs Fifteen
//! offers.

use common::board::{Grid, GridDimensions, Puzzle, TileCoord};
use common::ffi::{Preset, RawDrawing};
use common::menu::{self, LegendEntry, LegendGroup, TabSpec};
use common::session::{DrawHandle, Session};
use std::ffi::{CStr, c_char, c_int};

/// What a tile on the grid holds. The gap is the one square a
/// neighbouring tile can slide into.
#[derive(Clone, Copy, PartialEq, Eq)]
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

/// The mouse inputs Fifteen takes.
const MOUSE_CONTROLS: &[LegendEntry] =
    &[LegendEntry { input: "Left button", description: "slide tiles into the gap" }];

/// The keyboard inputs Fifteen takes.
const KEYBOARD_CONTROLS: &[LegendEntry] = &[
    LegendEntry { input: "Arrows", description: "slide the tile that way into the gap" },
    LegendEntry { input: "H", description: "make a suggested move" },
];

const GAME_CONTROLS: &[LegendGroup] = &[
    LegendGroup { label: Some("Mouse:"), entries: MOUSE_CONTROLS },
    LegendGroup { label: Some("Keyboard:"), entries: KEYBOARD_CONTROLS },
];

/// Fifteen's menu tabs, just the usuals since it doesn't have ones that
/// are specific to it.
pub(crate) fn tab_specs(presets: &[Preset], can_solve: bool) -> Vec<TabSpec<FifteenAction>> {
    menu::tabs(presets, can_solve, Vec::new(), GAME_CONTROLS)
}

/// Sends one mouse button press on the given tile, converting it into
/// the pixel coordinates fifteen.c's own click handling expects.
/// Returns `false` if it signalled quit.
pub(crate) fn click_tile(
    session: &mut Session<FifteenPuzzle>,
    (tile_x, tile_y): TileCoord,
    button: c_int,
) -> bool {
    let tilesize = session.tilesize();
    // fifteen.c's BORDER, half a tile around the grid.
    let border = tilesize / 2;

    // Centers the click in the tile.
    let pixel_x = border + tile_x as c_int * tilesize + tilesize / 2;
    let pixel_y = border + tile_y as c_int * tilesize + tilesize / 2;

    session.process_click(pixel_x, pixel_y, button)
}

/// The given tiles a click could slide. Those are the ones in the gap's
/// row or column, apart from the gap itself.
pub(crate) fn exclude_immovable(
    puzzle: &FifteenPuzzle,
    tile_coords: &[TileCoord],
) -> Vec<TileCoord> {
    let (gap_x, gap_y) = gap_position(puzzle);

    tile_coords
        .iter()
        .copied()
        .filter(|&(tile_x, tile_y)| (tile_x == gap_x) != (tile_y == gap_y))
        .collect()
}

/// Where the gap is on the grid.
pub(crate) fn gap_position(puzzle: &FifteenPuzzle) -> TileCoord {
    puzzle
        .tiles
        .iter()
        .enumerate()
        .find_map(|(tile_y, row)| {
            row.iter()
                .position(|&tile| tile == Tile::Gap)
                .map(|tile_x| (tile_x, tile_y))
        })
        .expect("a gap on the board")
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::input::LEFT_BUTTON;
    use std::time::Duration;

    /// Clicking the tile beside the gap slides it in, leaving the gap
    /// where the tile was, which only works if the click reached
    /// fifteen.c on that tile.
    #[test]
    fn clicking_a_tile_beside_the_gap_slides_it_in() {
        let mut session = Session::<FifteenPuzzle>::start();
        let (gap_x, gap_y) = gap_position(session.puzzle());
        let tile = if gap_x > 0 {
            (gap_x - 1, gap_y)
        } else {
            (gap_x + 1, gap_y)
        };

        click_tile(&mut session, tile, LEFT_BUTTON);

        assert_eq!(gap_position(session.puzzle()), tile);
    }

    /// Solving the puzzle starts the flash, which keeps the mid-end's
    /// timer on until the flash has had its time.
    #[test]
    fn solving_the_puzzle_runs_the_timer_until_the_flash_ends() {
        let mut session = Session::<FifteenPuzzle>::start();

        // Each 'h' plays a suggested move. Enough of them solve the
        // puzzle without counting as using the solver.
        for _ in 0..10_000 {
            if session.is_timer_active() {
                break;
            }
            session.process_key('h' as c_int);
        }
        assert!(session.puzzle().status.starts_with("COMPLETED"));
        assert!(session.is_timer_active());

        session.timer(Duration::from_secs(1));
        assert!(!session.is_timer_active());
    }

    /// The one preset's board, with one gap among the fifteen numbers.
    #[test]
    fn starting_board_is_four_by_four_with_one_gap() {
        let session = Session::<FifteenPuzzle>::start();
        let tiles = &session.puzzle().tiles;
        let gaps = tiles.iter().flatten().filter(|&&tile| tile == Tile::Gap).count();

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

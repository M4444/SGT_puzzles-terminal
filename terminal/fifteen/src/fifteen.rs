//! Fifteen's board and the tiles on it.

use common::board::{Grid, GridDimensions, Puzzle};

/// What a tile on the grid holds. The gap is the one square a
/// neighbouring tile can slide into.
#[derive(Clone, Copy)]
pub(crate) enum Tile {
    Number(u8),
    Gap,
}

pub(crate) type Tiles = Grid<Tile>;

/// Fifteen has no menu choices of its own.
#[derive(Clone, Copy)]
pub(crate) enum FifteenAction {}

/// A Fifteen puzzle.
pub(crate) struct FifteenPuzzle {
    pub tiles: Tiles,
}

impl Puzzle for FifteenPuzzle {
    fn dimensions(&self) -> GridDimensions {
        (self.tiles[0].len(), self.tiles.len())
    }
}

/// A hardcoded game in progress, with a zero standing for the gap.
pub(crate) fn hardcoded_game() -> FifteenPuzzle {
    /// Fifteen's only preset is four tiles across.
    const WIDTH: usize = 4;

    let values: [u8; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 0, 13, 14, 15, 12];
    let tiles = values
        .iter()
        .map(|&value| match value {
            0 => Tile::Gap,
            number => Tile::Number(number),
        })
        .collect::<Vec<_>>();

    FifteenPuzzle { tiles: tiles.chunks(WIDTH).map(|row| row.to_vec()).collect() }
}

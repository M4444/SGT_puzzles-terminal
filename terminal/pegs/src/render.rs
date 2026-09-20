//! Draws a Pegs board onto a canvas, its outline first, then its pegs
//! and holes and the cursor.

use crate::pegs::{Cursor, PegsPuzzle, Tile, Tiles};
use common::board::{center_mid, draw_irregular_grid_outline, tile_rect};
use common::canvas::{Canvas, Weight, draw_rect_outline};

const PEG: char = '⬤';
const HOLE: char = '◯';
/// A peg lifted off its tile, waiting for the jump to finish.
const PICKED_UP_PEG: char = '⨀';

pub(crate) fn draw_board(canvas: &mut Canvas, puzzle: &PegsPuzzle) {
    let tiles = &puzzle.tiles;

    draw_irregular_grid_outline(canvas, tiles, |&tile| tile != Tile::Obstacle);
    draw_pegs_and_holes(canvas, tiles, puzzle.cursor);
    draw_cursor(canvas, puzzle.cursor);
}

fn draw_pegs_and_holes(canvas: &mut Canvas, tiles: &Tiles, cursor: Cursor) {
    for (tile_y, row) in tiles.iter().enumerate() {
        for (tile_x, &tile) in row.iter().enumerate() {
            let tile_coord = (tile_x, tile_y);
            let picked_up = cursor.jumping && cursor.position == tile_coord;

            match tile {
                Tile::Hole => canvas.draw_char(center_mid(tile_coord), HOLE),
                Tile::Peg if picked_up => canvas.draw_char(center_mid(tile_coord), PICKED_UP_PEG),
                Tile::Peg => canvas.draw_char(center_mid(tile_coord), PEG),
                Tile::Obstacle => {}
            }
        }
    }
}

/// Draws a heavy box around the cursor's tile, if it's currently shown.
fn draw_cursor(canvas: &mut Canvas, cursor: Cursor) {
    if !cursor.visible {
        return;
    }

    draw_rect_outline(canvas, tile_rect(cursor.position), Weight::Heavy);
}

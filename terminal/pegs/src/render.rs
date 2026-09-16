//! Renders a Pegs board as a box-drawing string for the terminal.

use crate::pegs::{Cursor, PegsPuzzle, Tile, Tiles};
use common::board::{Board, center_mid, draw_frame, draw_irregular_grid_outline, tile_rect};
use common::canvas::{Canvas, Size, Weight, draw_rect_outline, flatten_to_lines};

const PEG: char = '⬤';
const HOLE: char = '◯';
/// A peg lifted off its tile, waiting for the jump to finish.
const PICKED_UP_PEG: char = '⨀';

pub(crate) fn render_game(
    puzzle: &PegsPuzzle,
    wants_status_bar: bool,
    terminal_columns: usize,
) -> String {
    let tiles = &puzzle.tiles;

    let board = Board::new((tiles[0].len(), tiles.len()), wants_status_bar);
    let mut canvas = Canvas::new(board.rect(), Size::new(0, 0));

    draw_irregular_grid_outline(&mut canvas, tiles, |&tile| tile != Tile::Obstacle);
    draw_pegs_and_holes(&mut canvas, tiles, puzzle.cursor);
    draw_cursor(&mut canvas, puzzle.cursor);
    draw_frame(&mut canvas, board.frame);

    flatten_to_lines(&canvas, terminal_columns).join("\n")
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

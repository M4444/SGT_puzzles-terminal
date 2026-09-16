//! Renders a Pegs board as a box-drawing string for the terminal.

use crate::pegs::{PegsPuzzle, Tile, Tiles};
use common::board::{Board, center_mid, draw_frame, draw_irregular_grid_outline};
use common::canvas::{Canvas, Size, flatten_to_lines};

const PEG: char = '⬤';
const HOLE: char = '◯';

pub(crate) fn render_game(
    puzzle: &PegsPuzzle,
    wants_status_bar: bool,
    terminal_columns: usize,
) -> String {
    let tiles = &puzzle.tiles;

    let board = Board::new((tiles[0].len(), tiles.len()), wants_status_bar);
    let mut canvas = Canvas::new(board.rect(), Size::new(0, 0));

    draw_irregular_grid_outline(&mut canvas, tiles, |&tile| tile != Tile::Obstacle);
    draw_pegs_and_holes(&mut canvas, tiles);
    draw_frame(&mut canvas, board.frame);

    flatten_to_lines(&canvas, terminal_columns).join("\n")
}

fn draw_pegs_and_holes(canvas: &mut Canvas, tiles: &Tiles) {
    for (tile_y, row) in tiles.iter().enumerate() {
        for (tile_x, &tile) in row.iter().enumerate() {
            match tile {
                Tile::Hole => canvas.draw_char(center_mid((tile_x, tile_y)), HOLE),
                Tile::Peg => canvas.draw_char(center_mid((tile_x, tile_y)), PEG),
                Tile::Obstacle => {}
            }
        }
    }
}

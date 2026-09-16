//! Draws a Pegs board for the terminal.

mod pegs;

use common::board::{Board, center_mid, draw_frame, draw_irregular_grid_outline};
use common::canvas::{Canvas, Size, flatten_to_lines};
use pegs::{Tile, Tiles};

const PEG: char = '⬤';
const HOLE: char = '◯';

fn main() {
    let session = pegs::new_session();
    let tiles = &session.puzzle().tiles;

    let board = Board::new((tiles[0].len(), tiles.len()));
    let mut canvas = Canvas::new(board.rect(), Size::new(0, 0));

    draw_irregular_grid_outline(&mut canvas, tiles, |&tile| tile != Tile::Obstacle);
    draw_pegs_and_holes(&mut canvas, tiles);
    draw_frame(&mut canvas, board.frame);

    for line in flatten_to_lines(&canvas, usize::MAX) {
        println!("{line}");
    }
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

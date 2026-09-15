//! Draws a hardcoded Pegs board for the terminal.

use common::board::{Board, Grid, center_mid, draw_frame, draw_irregular_grid_outline};
use common::canvas::{Canvas, Size, flatten_to_lines};

type Tiles = Grid<Tile>;

/// What a tile on the grid holds.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Tile {
    Hole,
    Peg,
    Obstacle,
}

const PEG: char = '⬤';
const HOLE: char = '◯';

fn main() {
    let tiles = build_board();

    let board = Board::new((tiles[0].len(), tiles.len()));
    let mut canvas = Canvas::new(board.rect(), Size::new(0, 0));

    draw_irregular_grid_outline(&mut canvas, &tiles, |&tile| tile != Tile::Obstacle);
    draw_pegs_and_holes(&mut canvas, &tiles);
    draw_frame(&mut canvas, board.frame);

    for line in flatten_to_lines(&canvas, usize::MAX) {
        println!("{line}");
    }
}

/// The 7x7 cross, with a peg on every tile on the board except the center.
fn build_board() -> Tiles {
    use Tile::{Hole, Obstacle, Peg};

    vec![
        vec![Obstacle, Obstacle, Peg, Peg, Peg, Obstacle, Obstacle],
        vec![Obstacle, Obstacle, Peg, Peg, Peg, Obstacle, Obstacle],
        vec![Peg, Peg, Peg, Peg, Peg, Peg, Peg],
        vec![Peg, Peg, Peg, Hole, Peg, Peg, Peg],
        vec![Peg, Peg, Peg, Peg, Peg, Peg, Peg],
        vec![Obstacle, Obstacle, Peg, Peg, Peg, Obstacle, Obstacle],
        vec![Obstacle, Obstacle, Peg, Peg, Peg, Obstacle, Obstacle],
    ]
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

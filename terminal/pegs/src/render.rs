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

#[cfg(test)]
mod tests {
    use super::*;
    use common::board::{Board, GridDimensions, left_side};
    use common::canvas::Size;

    /// A cross of three tiles, a peg either side of the hole in the
    /// middle, with the corners off the board.
    fn puzzle(cursor: Cursor) -> PegsPuzzle {
        let row = |tiles: [Tile; 3]| tiles.to_vec();

        PegsPuzzle {
            tiles: vec![
                row([Tile::Obstacle, Tile::Peg, Tile::Obstacle]),
                row([Tile::Peg, Tile::Hole, Tile::Peg]),
                row([Tile::Obstacle, Tile::Peg, Tile::Obstacle]),
            ],
            cursor,
        }
    }

    /// A cursor that isn't shown, so nothing draws it.
    fn hidden_cursor() -> Cursor {
        Cursor { position: (1, 1), visible: false, jumping: false }
    }

    fn canvas(dimensions: GridDimensions) -> Canvas {
        Canvas::new(Board::new(dimensions, false).rect(), Size::new(0, 0))
    }

    #[test]
    fn pegs_and_holes_are_drawn_at_tile_centres() {
        let mut canvas = canvas((3, 3));
        draw_board(&mut canvas, &puzzle(hidden_cursor()));

        assert_eq!(canvas.char_at(center_mid((1, 0))), PEG);
        assert_eq!(canvas.char_at(center_mid((1, 1))), HOLE);
        // A tile off the board holds nothing to draw.
        assert_eq!(canvas.char_at(center_mid((0, 0))), ' ');
    }

    #[test]
    fn only_the_picked_up_peg_is_drawn_lifted() {
        let jumping = Cursor { position: (1, 0), visible: true, jumping: true };
        let mut canvas = canvas((3, 3));
        draw_board(&mut canvas, &puzzle(jumping));

        assert_eq!(canvas.char_at(center_mid((1, 0))), PICKED_UP_PEG);
        assert_eq!(canvas.char_at(center_mid((0, 1))), PEG);
    }

    #[test]
    fn the_cursor_boxes_its_tile_only_while_it_is_shown() {
        let shown = Cursor { position: (1, 1), visible: true, jumping: false };
        let mut boxed = canvas((3, 3));
        let mut plain = canvas((3, 3));

        draw_board(&mut boxed, &puzzle(shown));
        draw_board(&mut plain, &puzzle(hidden_cursor()));

        assert_eq!(boxed.char_at(left_side((1, 1))), '┃');
        assert_ne!(plain.char_at(left_side((1, 1))), '┃');
    }
}

//! Draws a Fifteen board onto a canvas, its grid lines first, then the
//! number each tile carries.

use crate::fifteen::{FifteenPuzzle, Tile, Tiles};
use common::board::{Puzzle, TILE_CONTENT_WIDTH, center_left, draw_grid_lines};
use common::canvas::Canvas;

pub(crate) fn draw_board(canvas: &mut Canvas, puzzle: &FifteenPuzzle) {
    draw_grid_lines(canvas, puzzle.dimensions());
    draw_numbers(canvas, &puzzle.tiles);
}

/// Writes each number across its tile's content cells, centered, so a
/// two-digit number leans right. The gap has no number of its own to
/// draw.
fn draw_numbers(canvas: &mut Canvas, tiles: &Tiles) {
    for (tile_y, row) in tiles.iter().enumerate() {
        for (tile_x, &tile) in row.iter().enumerate() {
            let Tile::Number(number) = tile else {
                continue;
            };
            let text = centered(&number.to_string(), TILE_CONTENT_WIDTH);

            canvas.draw_text(center_left((tile_x, tile_y)), &text);
        }
    }
}

/// Centers `text` in `width` columns. When the padding can't be split
/// evenly, the larger half goes on the left, so the text leans right.
/// Text as wide as the space, or wider, comes back unchanged.
fn centered(text: &str, width: usize) -> String {
    let padding = width.saturating_sub(text.chars().count());
    let left = padding.div_ceil(2);

    format!("{}{text}{}", " ".repeat(left), " ".repeat(padding - left))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Padding that can't be split evenly leaves the text right of
    /// center.
    #[test]
    fn centered_text_leans_right() {
        assert_eq!(centered("9", 3), " 9 ");
        assert_eq!(centered("10", 3), " 10");
        assert_eq!(centered("10", 4), " 10 ");
        assert_eq!(centered("10", 5), "  10 ");
    }

    /// Text too wide to center comes back whole, since cutting it
    /// would lose what it says.
    #[test]
    fn centered_text_wider_than_the_space_is_left_alone() {
        assert_eq!(centered("100", 3), "100");
        assert_eq!(centered("1000", 3), "1000");
    }
}

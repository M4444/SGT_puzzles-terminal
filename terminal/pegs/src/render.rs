//! Renders a Pegs board as a box-drawing string for the terminal.

use crate::pegs::{Cursor, PegsAction, PegsPuzzle, Tile, Tiles};
use common::board::{Board, center_mid, draw_frame, draw_irregular_grid_outline, tile_rect};
use common::canvas::{Canvas, Weight, draw_rect_outline, flatten_to_lines};
use common::menu::{Menu, MenuState};

const PEG: char = '⬤';
const HOLE: char = '◯';
/// A peg lifted off its tile, waiting for the jump to finish.
const PICKED_UP_PEG: char = '⨀';

pub(crate) fn render_game(
    puzzle: &PegsPuzzle,
    wants_status_bar: bool,
    menu_state: MenuState,
    menu: &mut Menu<PegsAction>,
    terminal_columns: usize,
) -> String {
    let tiles = &puzzle.tiles;

    let board = Board::new((tiles[0].len(), tiles.len()), wants_status_bar);
    let board_rect = board.rect();
    let mut canvas = Canvas::new(board_rect, menu.size());

    draw_irregular_grid_outline(&mut canvas, tiles, |&tile| tile != Tile::Obstacle);
    draw_pegs_and_holes(&mut canvas, tiles, puzzle.cursor);
    draw_cursor(&mut canvas, puzzle.cursor);
    // The frame is what marks the board as focused.
    if !menu.has_focus() {
        draw_frame(&mut canvas, board.frame);
    }

    menu.set_placement(board_rect, terminal_columns);
    menu.draw(&mut canvas, menu_state, |action| match action {});

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

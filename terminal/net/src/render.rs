//! Renders a Net board as a box-drawing string for the terminal.
//!
//! Given a `NetPuzzle`, `render_game()` draws the board onto a `Canvas`
//! in phases (grid lines, wires and endpoints, source, barriers,
//! cursor, frame, status bar), adds the side menu, then flattens the
//! result to text.

use crate::net::{self, Cursor, NetAction, NetPuzzle, Tiles};
use common::board::{
    Board, GridDimensions, TileCoord, bottom_border_middle, bottom_left, bottom_mid, bottom_right,
    center_left, center_mid, center_right, draw_frame, draw_grid_lines, draw_status_bar, left_side,
    right_side, tile_content_coords, tile_full_coords, tile_rect, top_border_middle, top_left,
    top_mid, top_right,
};
use common::canvas::{Canvas, Mark, Weight, draw_line, draw_rect_outline, flatten_to_lines};
use common::menu::{Menu, MenuState};
use std::borrow::Cow;

pub(crate) fn render_game(
    puzzle: &NetPuzzle,
    wants_status_bar: bool,
    styles: Styles,
    menu_state: MenuState,
    menu: &mut Menu<NetAction>,
    terminal_columns: usize,
) -> String {
    let puzzle = puzzle_relative_to_origin(puzzle);
    let dimensions = puzzle.dimensions;

    let board = Board::new(dimensions, wants_status_bar);
    let board_rect = board.rect();
    let mut canvas = Canvas::new(board_rect, menu.size());

    draw_grid_lines(&mut canvas, dimensions);
    draw_wires_and_endpoints(&mut canvas, &puzzle.tiles);
    draw_source(&mut canvas, puzzle.source);
    draw_barriers(&mut canvas, &puzzle.tiles);
    draw_locked(&mut canvas, &puzzle.tiles, styles.lock);
    draw_cursor(&mut canvas, puzzle.cursor, styles.cursor);
    // The frame is what marks the board as focused.
    if !menu.has_focus() {
        draw_frame(&mut canvas, board.frame);
    }
    if let Some(start) = board.status_bar_start {
        draw_status_bar(&mut canvas, start, &puzzle.status);
    }

    menu.set_placement(board_rect, terminal_columns);
    menu.draw(&mut canvas, menu_state, |action| net::is_current(action, styles));

    flatten_to_lines(&canvas, terminal_columns).join("\n")
}

/// Transforms a puzzle's tiles, source, and cursor from game coordinates
/// into their position relative to the current origin.
/// Returns the puzzle unchanged if the origin is `(0, 0)`.
fn puzzle_relative_to_origin(puzzle: &NetPuzzle) -> Cow<'_, NetPuzzle> {
    let origin = puzzle.origin;
    if origin == (0, 0) {
        return Cow::Borrowed(puzzle);
    }

    let dimensions = puzzle.dimensions;
    Cow::Owned(NetPuzzle {
        dimensions,
        tiles: shift_tiles_by_origin(&puzzle.tiles, origin),
        cursor: cursor_relative_to_origin(puzzle.cursor, dimensions, origin),
        source: relative_to_origin(puzzle.source, dimensions, origin),
        origin,
        status: puzzle.status.clone(),
    })
}

/// Transforms a cursor's position from game coordinates into its
/// position relative to the current origin, leaving its visibility
/// unchanged.
fn cursor_relative_to_origin(
    cursor: Cursor,
    dimensions: GridDimensions,
    origin: TileCoord,
) -> Cursor {
    Cursor {
        position: relative_to_origin(cursor.position, dimensions, origin),
        visible: cursor.visible,
    }
}

/// Shifts `tiles` by `origin`, the same transform net.c's own redraw
/// applies for a moved viewport origin.
fn shift_tiles_by_origin(tiles: &Tiles, (origin_x, origin_y): TileCoord) -> Tiles {
    let height = tiles.len();
    let width = tiles[0].len();

    (0..height)
        .map(|viewport_y| {
            let y = (viewport_y + origin_y) % height;
            (0..width)
                .map(|viewport_x| {
                    let x = (viewport_x + origin_x) % width;
                    tiles[y][x]
                })
                .collect()
        })
        .collect()
}

/// Converts a game coordinate (as reported by the mid-end) into its
/// position relative to the current origin.
fn relative_to_origin(
    (x, y): TileCoord,
    (width, height): GridDimensions,
    (origin_x, origin_y): TileCoord,
) -> TileCoord {
    ((x + width - origin_x) % width, (y + height - origin_y) % height)
}

const SOURCE_MARKER: [char; 3] = ['▐', '🬰', '▌']; // ▐🬰▌
const POWERED_ENDPOINT_MARKER: [char; 3] = ['▐', '█', '▌']; // ▐█▌
const UNPOWERED_ENDPOINT_MARKER: [char; 3] = ['⢸', '⣿', '⡇']; // ⢸⣿⡇

/// Marks a tile's three center cells with three literal marker characters.
fn mark_tile(canvas: &mut Canvas, tile_coord: TileCoord, marker: [char; 3]) {
    canvas.draw_char(center_left(tile_coord), marker[0]);
    canvas.draw_char(center_mid(tile_coord), marker[1]);
    canvas.draw_char(center_right(tile_coord), marker[2]);
}

/// Draws the wires and endpoint markers on top of the grid lines, light
/// where unpowered and heavy where carrying power. A tile with exactly one
/// arm becomes an endpoint.
fn draw_wires_and_endpoints(canvas: &mut Canvas, tiles: &Tiles) {
    for (tile_y, tile_row) in tiles.iter().enumerate() {
        for (tile_x, tile) in tile_row.iter().enumerate() {
            let wires = tile.wires;
            let is_powered = tile.powered;
            let weight = if is_powered {
                Weight::Heavy
            } else {
                Weight::Light
            };
            let center = center_mid((tile_x, tile_y));
            let mut arm_count = 0;

            if wires.right {
                draw_line(canvas, center, right_side((tile_x, tile_y)), weight);
                arm_count += 1;
            }
            if wires.up {
                draw_line(canvas, top_mid((tile_x, tile_y)), center, weight);
                arm_count += 1;
            }
            if wires.left {
                draw_line(canvas, left_side((tile_x, tile_y)), center, weight);
                arm_count += 1;
            }
            if wires.down {
                draw_line(canvas, center, bottom_mid((tile_x, tile_y)), weight);
                arm_count += 1;
            }

            if arm_count == 1 {
                let marker = if is_powered {
                    POWERED_ENDPOINT_MARKER
                } else {
                    UNPOWERED_ENDPOINT_MARKER
                };
                mark_tile(canvas, (tile_x, tile_y), marker);
            }
        }
    }
}

/// Draws the source's own box over whatever else is at that coordinate,
/// always in its own style regardless of the tile's wire shape.
fn draw_source(canvas: &mut Canvas, source: TileCoord) {
    mark_tile(canvas, source, SOURCE_MARKER);
}

/// The board's two visual choices.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Styles {
    pub(crate) cursor: CursorStyle,
    pub(crate) lock: LockStyle,
}

/// How the keyboard cursor's tile is visually marked.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) enum CursorStyle {
    /// A heavy-weight box drawn around the tile's own border.
    #[default]
    Outline,
    /// Reverse video over just the tile's content cells.
    ReverseTileCenter,
    /// Reverse video over the entire tile: all four of its borders and
    /// its content.
    ReverseTileFull,
}

/// Highlights the keyboard cursor's tile, if it's currently shown.
fn draw_cursor(canvas: &mut Canvas, cursor: Cursor, style: CursorStyle) {
    if !cursor.visible {
        return;
    }

    match style {
        CursorStyle::Outline => {
            draw_rect_outline(canvas, tile_rect(cursor.position), Weight::Heavy);
        }
        CursorStyle::ReverseTileCenter => {
            canvas.toggle_reversed_region(tile_content_coords(cursor.position));
        }
        CursorStyle::ReverseTileFull => {
            canvas.toggle_reversed_region(tile_full_coords(cursor.position));
        }
    }
}

/// How a locked tile is visually marked.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[expect(
    clippy::enum_variant_names,
    reason = "matches CursorStyle's names for the same styles"
)]
pub(crate) enum LockStyle {
    /// Reverse video over just the tile's content cells.
    ReverseTileCenter,
    /// Reverse video over the entire tile: all four of its borders and
    /// its content.
    ReverseTileFull,
    /// Reverse video over the tile's content, plus each border whose far
    /// side is either another locked tile or outside the grid, leaving
    /// borders against an unlocked neighbour untouched.
    #[default]
    ReverseTileConnected,
}

/// Highlights every locked tile.
fn draw_locked(canvas: &mut Canvas, tiles: &Tiles, style: LockStyle) {
    match style {
        LockStyle::ReverseTileCenter => draw_locked_content(canvas, tiles),
        LockStyle::ReverseTileFull => draw_locked_full(canvas, tiles),
        LockStyle::ReverseTileConnected => {
            draw_locked_content(canvas, tiles);
            draw_locked_connections(canvas, tiles);
            draw_locked_junctions(canvas, tiles);
        }
    }
}

/// Reverses every locked tile's content cells.
fn draw_locked_content(canvas: &mut Canvas, tiles: &Tiles) {
    for (tile_y, tile_row) in tiles.iter().enumerate() {
        for (tile_x, tile) in tile_row.iter().enumerate() {
            if tile.locked {
                canvas.mark_region(tile_content_coords((tile_x, tile_y)), Mark::Reversed);
            }
        }
    }
}

/// Reverses every locked tile's entire footprint.
fn draw_locked_full(canvas: &mut Canvas, tiles: &Tiles) {
    for (tile_y, tile_row) in tiles.iter().enumerate() {
        for (tile_x, tile) in tile_row.iter().enumerate() {
            if tile.locked {
                canvas.mark_region(tile_full_coords((tile_x, tile_y)), Mark::Reversed);
            }
        }
    }
}

/// Reverses each border between two adjacent tiles, or against the
/// grid's edge, wherever every tile touching it is locked.
fn draw_locked_connections(canvas: &mut Canvas, tiles: &Tiles) {
    let height = tiles.len();

    for (tile_y, tile_row) in tiles.iter().enumerate() {
        let width = tile_row.len();

        for (tile_x, tile) in tile_row.iter().enumerate() {
            if !tile.locked {
                continue;
            }

            if tile_x == 0 || tile_row[tile_x - 1].locked {
                canvas.mark(left_side((tile_x, tile_y)), Mark::Reversed);
            }
            if tile_x == width - 1 {
                canvas.mark(right_side((tile_x, tile_y)), Mark::Reversed);
            }
            if tile_y == 0 || tiles[tile_y - 1][tile_x].locked {
                canvas.mark_region(top_border_middle((tile_x, tile_y)), Mark::Reversed);
            }
            if tile_y == height - 1 {
                canvas.mark_region(bottom_border_middle((tile_x, tile_y)), Mark::Reversed);
            }
        }
    }
}

/// Reverses each junction shared by up to four tiles wherever every tile
/// touching it is locked.
fn draw_locked_junctions(canvas: &mut Canvas, tiles: &Tiles) {
    let height = tiles.len();
    let width = tiles[0].len();

    for junction_y in 0..=height {
        for junction_x in 0..=width {
            let top_left_locked_or_edge =
                junction_x == 0 || junction_y == 0 || tiles[junction_y - 1][junction_x - 1].locked;
            let top_right_locked_or_edge =
                junction_x == width || junction_y == 0 || tiles[junction_y - 1][junction_x].locked;
            let bottom_left_locked_or_edge =
                junction_x == 0 || junction_y == height || tiles[junction_y][junction_x - 1].locked;
            let bottom_right_locked_or_edge =
                junction_x == width || junction_y == height || tiles[junction_y][junction_x].locked;

            if top_left_locked_or_edge
                && top_right_locked_or_edge
                && bottom_left_locked_or_edge
                && bottom_right_locked_or_edge
            {
                canvas.mark(top_left((junction_x, junction_y)), Mark::Reversed);
            }
        }
    }
}

/// Draws barrier walls, fixed obstacles blocking a wire connection, in
/// heavy weight across the border they occupy. A non-wrapping grid
/// carries barriers along its whole outer boundary, so that ring falls
/// out of the same tile data as any interior wall.
fn draw_barriers(canvas: &mut Canvas, tiles: &Tiles) {
    for (tile_y, tile_row) in tiles.iter().enumerate() {
        for (tile_x, tile) in tile_row.iter().enumerate() {
            let tile_coord = (tile_x, tile_y);
            let barriers = tile.barriers;

            if barriers.up {
                draw_line(canvas, top_left(tile_coord), top_right(tile_coord), Weight::Heavy);
            }
            if barriers.down {
                draw_line(canvas, bottom_left(tile_coord), bottom_right(tile_coord), Weight::Heavy);
            }
            if barriers.left {
                draw_line(canvas, top_left(tile_coord), bottom_left(tile_coord), Weight::Heavy);
            }
            if barriers.right {
                draw_line(canvas, top_right(tile_coord), bottom_right(tile_coord), Weight::Heavy);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::board::TileCoordNeighbors;
    use crossterm::style::{Attribute, SetAttribute};

    /// A canvas sized for the board and its menu. These tests only need
    /// the space the menu takes up, so it is built without presets.
    fn canvas(dimensions: GridDimensions) -> Canvas {
        Canvas::new(Board::new(dimensions, true).rect(), Menu::new(net::tab_specs(&[])).size())
    }

    #[test]
    fn render_game_does_not_panic_across_many_generated_boards() {
        for _ in 0..100 {
            let session = crate::net::new_session();
            let puzzle = session.puzzle();
            render_game(
                puzzle,
                true,
                Styles::default(),
                MenuState::default(),
                &mut Menu::new(net::tab_specs(&[])),
                usize::MAX,
            );
        }
    }

    #[test]
    fn draw_cursor_outline_boxes_the_tile() {
        let mut canvas = canvas((3, 3));
        draw_grid_lines(&mut canvas, (3, 3));
        let cursor = Cursor { position: (1, 1), visible: true };
        draw_cursor(&mut canvas, cursor, CursorStyle::Outline);

        assert_eq!(canvas.char_at(top_left((1, 1))), '╆');
        assert_eq!(canvas.char_at(top_left((2, 1))), '╅');
        assert_eq!(canvas.char_at(top_left((1, 2))), '╄');
        assert_eq!(canvas.char_at(top_left((2, 2))), '╃');
        assert_eq!(canvas.char_at(left_side((1, 1))), '┃');
    }

    #[test]
    fn draw_cursor_reverse_tile_center_only_marks_content() {
        let mut canvas = canvas((3, 3));
        let cursor = Cursor { position: (1, 1), visible: true };
        draw_cursor(&mut canvas, cursor, CursorStyle::ReverseTileCenter);

        assert!(canvas.is_reversed(center_left((1, 1))));
        assert!(canvas.is_reversed(center_mid((1, 1))));
        assert!(canvas.is_reversed(center_right((1, 1))));
        assert!(!canvas.is_reversed(top_left((1, 1))));
    }

    #[test]
    fn draw_cursor_reverse_tile_full_includes_shared_edge() {
        let mut canvas = canvas((3, 3));
        let cursor = Cursor { position: (1, 1), visible: true };
        draw_cursor(&mut canvas, cursor, CursorStyle::ReverseTileFull);

        assert!(canvas.is_reversed(top_left((1, 1))));
        assert!(canvas.is_reversed(center_mid((1, 1))));
        assert!(canvas.is_reversed(top_left((2, 1))));
        assert!(canvas.is_reversed(top_left((1, 2))));
    }

    /// A placed menu draws where it was placed. Beside a 5x5 board, whose
    /// frame ends at column 24, the divider sits at 26 and the tabs
    /// start at 28.
    #[test]
    fn menu_draws_where_it_is_placed() {
        let mut menu = Menu::new(net::tab_specs(&[]));
        // Menu Controls' header, opened so a legend is drawn too.
        menu.click((3, 6));

        let board_rect = Board::new((5, 5), true).rect();
        let mut canvas = Canvas::new(board_rect, menu.size());
        menu.set_placement(board_rect, usize::MAX);
        menu.draw(&mut canvas, MenuState::default(), |action| {
            net::is_current(action, Styles::default())
        });

        // The divider, ending on the board's last row, then the first
        // button's corner, Cursor Style's header line and the legend's
        // first label.
        assert_eq!(canvas.char_at((26, 5)), '│');
        assert_eq!(canvas.char_at((26, 13)), '╵');
        assert_eq!(canvas.char_at((26, 14)), ' ');
        assert_eq!(canvas.char_at((28, 0)), '┌');
        assert_eq!(canvas.char_at((29, 4)), '─');
        assert_eq!(canvas.char_at((28, 7)), 'G');
    }

    /// A header's line and arrow stop at the terminal's edge when the
    /// menu runs past it.
    #[test]
    fn headers_stop_at_the_terminal_edge() {
        let mut menu = Menu::new(net::tab_specs(&[]));
        let board_rect = Board::new((5, 5), true).rect();
        let mut canvas = Canvas::new(board_rect, menu.size());

        // Type's header would run to column 84 in a terminal wide
        // enough, but this one is 60 columns wide.
        menu.set_placement(board_rect, 60);
        menu.draw(&mut canvas, MenuState::default(), |action| {
            net::is_current(action, Styles::default())
        });

        assert_eq!(canvas.char_at((59, 3)), '▼');
        assert_eq!(canvas.char_at((60, 3)), ' ');
    }

    /// An unavailable action's button is dimmed, so a rendered game
    /// carries the dim attribute only when one of them is unavailable.
    #[test]
    fn render_game_dims_unavailable_menu_buttons() {
        let session = crate::net::new_session();
        let puzzle = session.puzzle();
        let dim = SetAttribute(Attribute::Dim).to_string();
        let mut menu = Menu::new(net::tab_specs(&[]));
        let mut render = |menu_state| {
            render_game(puzzle, true, Styles::default(), menu_state, &mut menu, usize::MAX)
        };

        let fresh = MenuState { can_undo: false, can_redo: false, ..MenuState::default() };
        let mid_game = MenuState { can_undo: true, can_redo: true, ..MenuState::default() };

        assert!(render(fresh).contains(&dim));
        assert!(!render(mid_game).contains(&dim));
    }

    fn grid_with_locked((width, height): GridDimensions, locked_positions: &[TileCoord]) -> Tiles {
        let mut tiles = vec![vec![crate::net::Tile::default(); width]; height];
        for &(x, y) in locked_positions {
            tiles[y][x].locked = true;
        }
        tiles
    }

    #[test]
    fn draw_locked_connected_reverses_border_shared_with_another_locked_tile() {
        let dimensions = (4, 3);
        let target_tile = (1, 1);
        let tiles = grid_with_locked(dimensions, &[target_tile, target_tile.right()]);
        let mut canvas = canvas(dimensions);
        draw_locked(&mut canvas, &tiles, LockStyle::ReverseTileConnected);

        assert!(canvas.is_reversed(right_side(target_tile)));
    }

    #[test]
    fn draw_locked_connected_does_not_reverse_border_against_an_unlocked_neighbour() {
        let dimensions = (4, 3);
        let target_tile = (1, 1);
        let tiles = grid_with_locked(dimensions, &[target_tile, target_tile.right()]);
        let mut canvas = canvas(dimensions);
        draw_locked(&mut canvas, &tiles, LockStyle::ReverseTileConnected);

        assert!(!canvas.is_reversed(top_mid(target_tile)));
    }

    #[test]
    fn draw_locked_connected_reverses_border_against_the_grid_edge() {
        let dimensions = (3, 3);
        let target_tile = (0, 0);
        let tiles = grid_with_locked(dimensions, &[target_tile]);
        let mut canvas = canvas(dimensions);
        draw_locked(&mut canvas, &tiles, LockStyle::ReverseTileConnected);

        assert!(canvas.is_reversed(top_mid(target_tile)));
        assert!(canvas.is_reversed(left_side(target_tile)));
        assert!(!canvas.is_reversed(right_side(target_tile)));
    }

    /// Only a tile in the last row closes its bottom border against the
    /// grid's edge.
    #[test]
    fn draw_locked_connected_reverses_border_against_the_bottom_edge() {
        let dimensions = (3, 3);
        let target_tile = (0, 2);
        let tiles = grid_with_locked(dimensions, &[target_tile, (1, 1)]);
        let mut canvas = canvas(dimensions);
        draw_locked(&mut canvas, &tiles, LockStyle::ReverseTileConnected);

        assert!(bottom_border_middle(target_tile).all(|coord| canvas.is_reversed(coord)));
        assert!(!bottom_border_middle((1, 1)).any(|coord| canvas.is_reversed(coord)));
    }

    /// Only a tile in the last column closes its right border against the
    /// grid's edge.
    #[test]
    fn draw_locked_connected_reverses_border_against_the_right_edge() {
        let dimensions = (3, 3);
        let target_tile = (2, 0);
        let tiles = grid_with_locked(dimensions, &[target_tile, (1, 1)]);
        let mut canvas = canvas(dimensions);
        draw_locked(&mut canvas, &tiles, LockStyle::ReverseTileConnected);

        assert!(canvas.is_reversed(right_side(target_tile)));
        assert!(!canvas.is_reversed(right_side((1, 1))));
    }

    #[test]
    fn draw_locked_connected_forms_a_solid_rectangle_for_a_horizontal_pair() {
        let dimensions = (4, 3);
        let target_tile = (1, 1);
        let tiles = grid_with_locked(dimensions, &[target_tile, target_tile.right()]);
        let mut canvas = canvas(dimensions);
        draw_locked(&mut canvas, &tiles, LockStyle::ReverseTileConnected);

        // The corners of the shared border also touch the unlocked
        // tiles above and below, and stay untouched.
        assert!(!canvas.is_reversed(top_right(target_tile)));
        assert!(!canvas.is_reversed(bottom_right(target_tile)));
    }

    #[test]
    fn draw_locked_connected_closes_the_interior_corner_of_a_solid_block() {
        let dimensions = (3, 3);
        let target_tile = (0, 0);
        let tiles = grid_with_locked(
            dimensions,
            &[
                target_tile,
                target_tile.right(),
                target_tile.bottom(),
                target_tile.right().bottom(),
            ],
        );
        let mut canvas = canvas(dimensions);
        draw_locked(&mut canvas, &tiles, LockStyle::ReverseTileConnected);

        assert!(canvas.is_reversed(bottom_right(target_tile)));
    }

    #[test]
    fn draw_cursor_draws_nothing_when_not_visible() {
        let mut canvas = canvas((3, 3));
        let cursor = Cursor { position: (1, 1), visible: false };
        draw_cursor(&mut canvas, cursor, CursorStyle::default());

        assert!(!canvas.is_reversed(top_left((1, 1))));
        assert!(!canvas.is_reversed(center_mid((1, 1))));
    }

    /// A tile's `right` barrier draws on the edge it names and nowhere
    /// else. The two corners it doesn't touch stay the plain light cross
    /// `draw_grid_lines` left there.
    #[test]
    fn draw_barriers_draws_on_the_named_edge_only() {
        let dimensions = (3, 3);
        let mut tiles = vec![vec![crate::net::Tile::default(); 3]; 3];
        tiles[1][1].barriers.right = true;

        let mut canvas = canvas(dimensions);
        draw_grid_lines(&mut canvas, dimensions);
        draw_barriers(&mut canvas, &tiles);

        assert_eq!(canvas.char_at(top_right((1, 1))), '╁');
        assert_eq!(canvas.char_at(bottom_right((1, 1))), '╀');
        assert_eq!(canvas.char_at(top_left((1, 1))), '┼');
        assert_eq!(canvas.char_at(bottom_left((1, 1))), '┼');
    }

    /// A corner tile's two boundary barriers, `up` and `left`, both end
    /// at its top-left corner. That corner becomes a solid heavy angle,
    /// and the opposite corner, which neither barrier reaches, stays the
    /// plain light cross.
    #[test]
    fn draw_barriers_combines_two_barriers_meeting_at_a_corner() {
        let dimensions = (3, 3);
        let mut tiles = vec![vec![crate::net::Tile::default(); 3]; 3];
        tiles[0][0].barriers.up = true;
        tiles[0][0].barriers.left = true;

        let mut canvas = canvas(dimensions);
        draw_grid_lines(&mut canvas, dimensions);
        draw_barriers(&mut canvas, &tiles);

        assert_eq!(canvas.char_at(top_left((0, 0))), '┏');
        assert_eq!(canvas.char_at(bottom_right((0, 0))), '┼');
    }

    #[test]
    fn draw_barriers_draws_nothing_when_no_tile_carries_one() {
        let dimensions = (3, 3);
        let tiles = vec![vec![crate::net::Tile::default(); 3]; 3];

        let mut actual = canvas(dimensions);
        draw_grid_lines(&mut actual, dimensions);
        draw_barriers(&mut actual, &tiles);

        let mut expected = canvas(dimensions);
        draw_grid_lines(&mut expected, dimensions);

        assert_eq!(flatten_to_lines(&actual, usize::MAX), flatten_to_lines(&expected, usize::MAX));
    }
}

//! Where a grid of tiles, the frame around it and the status bar below
//! it sit on the canvas.

use crate::canvas::{
    CANVAS_MARGIN_X, CANVAS_MARGIN_Y, Canvas, Coord, Rect, Size, Weight, draw_line,
    draw_rect_outline,
};

/// A grid of tiles, indexed `[row][column]`.
pub type Grid<T> = Vec<Vec<T>>;
pub type GridDimensions = (usize, usize);
pub type TileCoord = (usize, usize);

/// A tile coordinate's neighbours one step over in each direction. The
/// checked versions give `None` where the step would leave `usize`'s
/// range, so from the top row there's no tile above.
pub trait TileCoordNeighbors {
    fn right(&self) -> TileCoord;
    fn top(&self) -> TileCoord;
    fn left(&self) -> TileCoord;
    fn bottom(&self) -> TileCoord;

    fn checked_right(&self) -> Option<TileCoord>;
    fn checked_top(&self) -> Option<TileCoord>;
    fn checked_left(&self) -> Option<TileCoord>;
    fn checked_bottom(&self) -> Option<TileCoord>;
}

impl TileCoordNeighbors for TileCoord {
    fn right(&self) -> TileCoord {
        let (x, y) = *self;
        (x + 1, y)
    }
    fn top(&self) -> TileCoord {
        let (x, y) = *self;
        (x, y - 1)
    }
    fn left(&self) -> TileCoord {
        let (x, y) = *self;
        (x - 1, y)
    }
    fn bottom(&self) -> TileCoord {
        let (x, y) = *self;
        (x, y + 1)
    }

    fn checked_right(&self) -> Option<TileCoord> {
        let (x, y) = *self;
        Some((x.checked_add(1)?, y))
    }
    fn checked_top(&self) -> Option<TileCoord> {
        let (x, y) = *self;
        Some((x, y.checked_sub(1)?))
    }
    fn checked_left(&self) -> Option<TileCoord> {
        let (x, y) = *self;
        Some((x.checked_sub(1)?, y))
    }
    fn checked_bottom(&self) -> Option<TileCoord> {
        let (x, y) = *self;
        Some((x, y.checked_add(1)?))
    }
}

type Offset = (usize, usize);

/// Gap between the grid lines and the frame: FRAME_MARGIN_X columns on each
/// side, FRAME_MARGIN_Y rows above and below. Terminal character cells are
/// taller than they are wide, so the vertical gap grows at half the rate of
/// the horizontal one to look even.
const FRAME_MARGIN_X: usize = 1;
const FRAME_MARGIN_Y: usize = FRAME_MARGIN_X / 2;

const GRID_OFFSET_X: usize = CANVAS_MARGIN_X + FRAME_MARGIN_X + 1;
const GRID_OFFSET_Y: usize = CANVAS_MARGIN_Y + FRAME_MARGIN_Y + 1;

/// Columns per tile: two shared border columns plus three center columns.
const TILE_WIDTH: usize = 5;
/// Rows per tile: two shared border rows plus one center row.
const TILE_HEIGHT: usize = 3;

/// Addressing a tile's contents can be done relative to the tile. The tile
/// is given as `(tile_x, tile_y)`, and the offset within it as a
/// quarter-step `offset_x` across (0..TILE_WIDTH) and a half-step
/// `offset_y` down (0..TILE_HEIGHT), overlapping by one step with each
/// neighbouring tile to merge borders between them.
fn tile_offset_to_coord((tile_x, tile_y): TileCoord, (offset_x, offset_y): Offset) -> Coord {
    let x = GRID_OFFSET_X + (TILE_WIDTH - 1) * tile_x + offset_x;
    let y = GRID_OFFSET_Y + (TILE_HEIGHT - 1) * tile_y + offset_y;

    Coord::new(x, y)
}

/// The inverse of `tile_offset_to_coord`: which tile a screen coordinate
/// falls in, and its offset within that tile, or `None` if the
/// coordinate is outside the grid entirely.
fn coord_to_tile_offset(
    coord: Coord,
    (width, height): GridDimensions,
) -> Option<(TileCoord, Offset)> {
    // Left of or above the grid.
    if coord.x < GRID_OFFSET_X || coord.y < GRID_OFFSET_Y {
        return None;
    }

    let relative_x = coord.x - GRID_OFFSET_X;
    let relative_y = coord.y - GRID_OFFSET_Y;

    // Right of or below the grid.
    if relative_x >= (TILE_WIDTH - 1) * width || relative_y >= (TILE_HEIGHT - 1) * height {
        return None;
    }

    let tile = (relative_x / (TILE_WIDTH - 1), relative_y / (TILE_HEIGHT - 1));
    let offset = (relative_x % (TILE_WIDTH - 1), relative_y % (TILE_HEIGHT - 1));

    Some((tile, offset))
}

/// Every tile whose full footprint (borders included) contains a
/// screen coordinate: just the one tile for a content cell or an
/// outer grid edge, two for a border shared between neighbours, or up
/// to four for a junction.
pub fn tiles_at(position: impl Into<Coord>, dimensions: GridDimensions) -> Vec<TileCoord> {
    let Some((tile, (offset_x, offset_y))) = coord_to_tile_offset(position.into(), dimensions)
    else {
        return Vec::new();
    };

    let (tile_x, tile_y) = tile;

    let on_border_column = offset_x == 0;
    let on_border_row = offset_y == 0;

    // Center, or the single tile touched at an outer grid edge.
    let mut tiles = vec![tile];
    // Vertical border: shared with the tile to the left.
    if on_border_column && tile_x > 0 {
        tiles.push(tile.left());
    }
    // Horizontal border: shared with the tile above.
    if on_border_row && tile_y > 0 {
        tiles.push(tile.top());
    }
    // Junction: also shared with the tile above and to the left.
    if on_border_column && on_border_row && tile_x > 0 && tile_y > 0 {
        tiles.push(tile.left().top());
    }

    tiles
}

// Helper functions name specific coordinates within a tile's footprint:
//
//               top_mid    a = center_left
//                  │       b = center_mid
//                  ▼       c = center_right
// top_left ─────►┌───┐◄─── top_right
// left_side ────►│abc│◄─── right_side
// bottom_left ──►└───┘◄─── bottom_right
//                  ▲
//                  │
//             bottom_mid
//
pub fn top_left(tile_coord: TileCoord) -> Coord {
    tile_offset_to_coord(tile_coord, (0, 0))
}
pub fn top_mid(tile_coord: TileCoord) -> Coord {
    tile_offset_to_coord(tile_coord, (2, 0))
}
pub fn top_right(tile_coord: TileCoord) -> Coord {
    tile_offset_to_coord(tile_coord, (TILE_WIDTH - 1, 0))
}
pub fn left_side(tile_coord: TileCoord) -> Coord {
    tile_offset_to_coord(tile_coord, (0, 1))
}
pub fn center_left(tile_coord: TileCoord) -> Coord {
    tile_offset_to_coord(tile_coord, (1, 1))
}
pub fn center_mid(tile_coord: TileCoord) -> Coord {
    tile_offset_to_coord(tile_coord, (2, 1))
}
pub fn center_right(tile_coord: TileCoord) -> Coord {
    tile_offset_to_coord(tile_coord, (3, 1))
}
pub fn right_side(tile_coord: TileCoord) -> Coord {
    tile_offset_to_coord(tile_coord, (TILE_WIDTH - 1, 1))
}
pub fn bottom_left(tile_coord: TileCoord) -> Coord {
    tile_offset_to_coord(tile_coord, (0, TILE_HEIGHT - 1))
}
pub fn bottom_mid(tile_coord: TileCoord) -> Coord {
    tile_offset_to_coord(tile_coord, (2, TILE_HEIGHT - 1))
}
pub fn bottom_right(tile_coord: TileCoord) -> Coord {
    tile_offset_to_coord(tile_coord, (TILE_WIDTH - 1, TILE_HEIGHT - 1))
}

pub fn tile_rect(tile: TileCoord) -> Rect {
    Rect::from_corners(top_left(tile), bottom_right(tile))
}

/// The tile's three content coordinates.
pub fn tile_content_coords(tile: TileCoord) -> impl Iterator<Item = Coord> {
    tile_rect(tile).interior().coords()
}

/// The tile's entire footprint: all four of its borders and its content.
pub fn tile_full_coords(tile: TileCoord) -> impl Iterator<Item = Coord> {
    tile_rect(tile).coords()
}

/// The tile's top border, excluding its left and right corners.
pub fn top_border_middle(tile: TileCoord) -> impl Iterator<Item = Coord> {
    Rect::new(tile_offset_to_coord(tile, (1, 0)), Size::new(TILE_WIDTH - 2, 1)).coords()
}

/// The tile's bottom border, excluding its left and right corners.
pub fn bottom_border_middle(tile: TileCoord) -> impl Iterator<Item = Coord> {
    Rect::new(tile_offset_to_coord(tile, (1, TILE_HEIGHT - 1)), Size::new(TILE_WIDTH - 2, 1))
        .coords()
}

/// The board consists of the frame enclosing the grid and the status
/// bar on the row beneath it.
#[derive(Clone, Copy)]
pub struct Board {
    pub frame: Rect,
    pub status_bar_start: Coord,
}

impl Board {
    pub fn new((width, height): GridDimensions) -> Board {
        assert!(width > 0 && height > 0, "Board::new() called with a zero-tile board");

        let grid = Rect::from_corners(top_left((0, 0)), bottom_right((width - 1, height - 1)));

        // The frame's border sits FRAME_MARGIN cells clear of the grid
        // on every side.
        let frame = Rect::from_corners(
            Coord::new(grid.left() - FRAME_MARGIN_X - 1, grid.top() - FRAME_MARGIN_Y - 1),
            Coord::new(grid.right() + FRAME_MARGIN_X + 1, grid.bottom() + FRAME_MARGIN_Y + 1),
        );
        let status_bar_start = Coord::new(frame.left(), frame.bottom() + 1);

        Board { frame, status_bar_start }
    }

    /// The whole board, from the frame's top left to the end of the
    /// status bar's row.
    pub fn rect(self) -> Rect {
        let top_left = Coord::new(self.frame.left(), self.frame.top());
        let bottom_right = Coord::new(self.frame.right(), self.status_bar_start.y);
        Rect::from_corners(top_left, bottom_right)
    }
}

/// Draws the fixed tile borders between cells, independent of any wire.
pub fn draw_grid_lines(canvas: &mut Canvas, (width, height): GridDimensions) {
    for tile_y in 0..=height {
        draw_line(canvas, top_left((0, tile_y)), top_left((width, tile_y)), Weight::Light);
    }

    for tile_x in 0..=width {
        draw_line(canvas, top_left((tile_x, 0)), top_left((tile_x, height)), Weight::Light);
    }
}

/// Draws the outline around the tiles that are on the board.
pub fn draw_irregular_grid_outline<T>(
    canvas: &mut Canvas,
    grid: &Grid<T>,
    is_on_board: impl Fn(&T) -> bool,
) {
    let is_on_board_at = |neighbour: Option<TileCoord>| {
        neighbour.is_some_and(|(tile_x, tile_y)| {
            grid.get(tile_y)
                .and_then(|row| row.get(tile_x))
                .is_some_and(&is_on_board)
        })
    };

    for (tile_y, row) in grid.iter().enumerate() {
        for (tile_x, tile) in row.iter().enumerate() {
            if !is_on_board(tile) {
                continue;
            }

            let tile_coord = (tile_x, tile_y);

            if !is_on_board_at(tile_coord.checked_right()) {
                draw_line(canvas, top_right(tile_coord), bottom_right(tile_coord), Weight::Light);
            }
            if !is_on_board_at(tile_coord.checked_top()) {
                draw_line(canvas, top_left(tile_coord), top_right(tile_coord), Weight::Light);
            }
            if !is_on_board_at(tile_coord.checked_left()) {
                draw_line(canvas, top_left(tile_coord), bottom_left(tile_coord), Weight::Light);
            }
            if !is_on_board_at(tile_coord.checked_bottom()) {
                draw_line(canvas, bottom_left(tile_coord), bottom_right(tile_coord), Weight::Light);
            }
        }
    }
}

const FRAME_WEIGHT: Weight = Weight::Double;

/// Draws the outer presentation frame, offset from the grid lines by
/// FRAME_MARGIN_X/FRAME_MARGIN_Y and from the canvas edge by
/// CANVAS_MARGIN_X/CANVAS_MARGIN_Y.
pub fn draw_frame(canvas: &mut Canvas, frame: Rect) {
    draw_rect_outline(canvas, frame, FRAME_WEIGHT);
}

pub fn draw_status_bar(canvas: &mut Canvas, start: Coord, status: &str) {
    canvas.draw_text(start, status);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_tile_boundary_shared(
        tile: TileCoord,
        next_tile: TileCoord,
        full_tile_offset: Offset,
    ) {
        let zero_offset: Offset = (0, 0);
        let end_of_tile = tile_offset_to_coord(tile, full_tile_offset);
        let start_of_next_tile = tile_offset_to_coord(next_tile, zero_offset);
        assert_eq!(end_of_tile, start_of_next_tile);
    }

    #[test]
    fn tile_boundary_shared_x() {
        let tile = (0, 0);
        assert_tile_boundary_shared(tile, tile.right(), (TILE_WIDTH - 1, 0));
    }

    #[test]
    fn tile_boundary_shared_y() {
        let tile = (0, 0);
        assert_tile_boundary_shared(tile, tile.bottom(), (0, TILE_HEIGHT - 1));
    }

    #[test]
    #[should_panic(expected = "Board::new() called with a zero-tile board")]
    fn board_new_panics_on_zero_tile_board() {
        Board::new((0, 0));
    }

    /// An L of three tiles on the board in a 2x2 grid. The border the two
    /// top tiles share stays blank and the outline turns a corner where
    /// it steps in around the bottom right tile, which is off the board.
    #[test]
    fn draw_irregular_grid_outline_skips_borders_between_tiles() {
        let grid = vec![vec![true, true], vec![true, false]];
        let mut canvas = Canvas::new(Board::new((2, 2)).rect(), Size::new(0, 0));

        draw_irregular_grid_outline(&mut canvas, &grid, |&is_on_board| is_on_board);

        assert_eq!(canvas.char_at(right_side((0, 0))), ' ');
        assert_eq!(canvas.char_at(top_right((0, 1))), '┌');
    }
}

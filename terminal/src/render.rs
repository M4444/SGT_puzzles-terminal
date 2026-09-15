//! Renders a Net board as a box-drawing string for the terminal.
//!
//! Given a `NetPuzzle`, `render_game()` draws the board onto a `Canvas`
//! in phases (grid lines, wires and endpoints, source, barriers,
//! cursor, frame, status bar), adds the side menu, then flattens the
//! result to text. Given a screen coordinate, `tiles_at()` locates
//! every tile it falls within.

use crate::menu::{ActionAvailability, Menu};
use crate::net::{Cursor, GridDimensions, NetPuzzle, TileCoord, TileCoordNeighbors, Tiles};
use crossterm::style::{Attribute, SetAttribute};
use std::borrow::Cow;
use std::cmp::max;

pub(crate) fn render_game(
    puzzle: &NetPuzzle,
    styles: Styles,
    menu: &mut Menu,
    availability: ActionAvailability,
    terminal_columns: usize,
) -> String {
    let puzzle = puzzle_relative_to_origin(puzzle);
    let dimensions = puzzle.dimensions;

    let board = Board::new(dimensions);
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
    draw_status_bar(&mut canvas, board.status_bar_start, &puzzle.status);

    menu.set_placement(board_rect, terminal_columns);
    menu.draw(&mut canvas, availability, styles);

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

/// How strongly a line is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Weight {
    None,
    Light,
    Heavy,
    Double,
}

/// Per-arm line weight, order (right, up, left, down).
type Code = [Weight; 4];

const BLANK: Code = [Weight::None; 4];

/// Elementwise max of two weight codes. The stronger arm wins wherever
/// two draws overlap.
fn combine(a: Code, b: Code) -> Code {
    [max(a[0], b[0]), max(a[1], b[1]), max(a[2], b[2]), max(a[3], b[3])]
}

/// Returns the Unicode glyph based on the arm-weight combination.
fn glyph(code: Code) -> char {
    match code {
        [Weight::None, Weight::None, Weight::None, Weight::None] => ' ',
        [Weight::None, Weight::None, Weight::None, Weight::Light] => '╷',
        [Weight::None, Weight::None, Weight::None, Weight::Heavy] => '╻',
        [Weight::None, Weight::None, Weight::Light, Weight::None] => '╴',
        [Weight::None, Weight::None, Weight::Light, Weight::Light] => '┐',
        [Weight::None, Weight::None, Weight::Light, Weight::Heavy] => '┒',
        [Weight::None, Weight::None, Weight::Light, Weight::Double] => '╖',
        [Weight::None, Weight::None, Weight::Heavy, Weight::None] => '╸',
        [Weight::None, Weight::None, Weight::Heavy, Weight::Light] => '┑',
        [Weight::None, Weight::None, Weight::Heavy, Weight::Heavy] => '┓',
        [Weight::None, Weight::None, Weight::Double, Weight::Light] => '╕',
        [Weight::None, Weight::None, Weight::Double, Weight::Double] => '╗',
        [Weight::None, Weight::Light, Weight::None, Weight::None] => '╵',
        [Weight::None, Weight::Light, Weight::None, Weight::Light] => '│',
        [Weight::None, Weight::Light, Weight::None, Weight::Heavy] => '╽',
        [Weight::None, Weight::Light, Weight::Light, Weight::None] => '┘',
        [Weight::None, Weight::Light, Weight::Light, Weight::Light] => '┤',
        [Weight::None, Weight::Light, Weight::Light, Weight::Heavy] => '┧',
        [Weight::None, Weight::Light, Weight::Heavy, Weight::None] => '┙',
        [Weight::None, Weight::Light, Weight::Heavy, Weight::Light] => '┥',
        [Weight::None, Weight::Light, Weight::Heavy, Weight::Heavy] => '┪',
        [Weight::None, Weight::Light, Weight::Double, Weight::None] => '╛',
        [Weight::None, Weight::Light, Weight::Double, Weight::Light] => '╡',
        [Weight::None, Weight::Heavy, Weight::None, Weight::None] => '╹',
        [Weight::None, Weight::Heavy, Weight::None, Weight::Light] => '╿',
        [Weight::None, Weight::Heavy, Weight::None, Weight::Heavy] => '┃',
        [Weight::None, Weight::Heavy, Weight::Light, Weight::None] => '┚',
        [Weight::None, Weight::Heavy, Weight::Light, Weight::Light] => '┦',
        [Weight::None, Weight::Heavy, Weight::Light, Weight::Heavy] => '┨',
        [Weight::None, Weight::Heavy, Weight::Heavy, Weight::None] => '┛',
        [Weight::None, Weight::Heavy, Weight::Heavy, Weight::Light] => '┩',
        [Weight::None, Weight::Heavy, Weight::Heavy, Weight::Heavy] => '┫',
        [Weight::None, Weight::Double, Weight::None, Weight::Double] => '║',
        [Weight::None, Weight::Double, Weight::Light, Weight::None] => '╜',
        [Weight::None, Weight::Double, Weight::Light, Weight::Double] => '╢',
        [Weight::None, Weight::Double, Weight::Double, Weight::None] => '╝',
        [Weight::None, Weight::Double, Weight::Double, Weight::Double] => '╣',
        [Weight::Light, Weight::None, Weight::None, Weight::None] => '╶',
        [Weight::Light, Weight::None, Weight::None, Weight::Light] => '┌',
        [Weight::Light, Weight::None, Weight::None, Weight::Heavy] => '┎',
        [Weight::Light, Weight::None, Weight::None, Weight::Double] => '╓',
        [Weight::Light, Weight::None, Weight::Light, Weight::None] => '─',
        [Weight::Light, Weight::None, Weight::Light, Weight::Light] => '┬',
        [Weight::Light, Weight::None, Weight::Light, Weight::Heavy] => '┰',
        [Weight::Light, Weight::None, Weight::Light, Weight::Double] => '╥',
        [Weight::Light, Weight::None, Weight::Heavy, Weight::None] => '╾',
        [Weight::Light, Weight::None, Weight::Heavy, Weight::Light] => '┭',
        [Weight::Light, Weight::None, Weight::Heavy, Weight::Heavy] => '┱',
        [Weight::Light, Weight::Light, Weight::None, Weight::None] => '└',
        [Weight::Light, Weight::Light, Weight::None, Weight::Light] => '├',
        [Weight::Light, Weight::Light, Weight::None, Weight::Heavy] => '┟',
        [Weight::Light, Weight::Light, Weight::Light, Weight::None] => '┴',
        [Weight::Light, Weight::Light, Weight::Light, Weight::Light] => '┼',
        [Weight::Light, Weight::Light, Weight::Light, Weight::Heavy] => '╁',
        [Weight::Light, Weight::Light, Weight::Heavy, Weight::None] => '┵',
        [Weight::Light, Weight::Light, Weight::Heavy, Weight::Light] => '┽',
        [Weight::Light, Weight::Light, Weight::Heavy, Weight::Heavy] => '╅',
        [Weight::Light, Weight::Heavy, Weight::None, Weight::None] => '┖',
        [Weight::Light, Weight::Heavy, Weight::None, Weight::Light] => '┞',
        [Weight::Light, Weight::Heavy, Weight::None, Weight::Heavy] => '┠',
        [Weight::Light, Weight::Heavy, Weight::Light, Weight::None] => '┸',
        [Weight::Light, Weight::Heavy, Weight::Light, Weight::Light] => '╀',
        [Weight::Light, Weight::Heavy, Weight::Light, Weight::Heavy] => '╂',
        [Weight::Light, Weight::Heavy, Weight::Heavy, Weight::None] => '┹',
        [Weight::Light, Weight::Heavy, Weight::Heavy, Weight::Light] => '╃',
        [Weight::Light, Weight::Heavy, Weight::Heavy, Weight::Heavy] => '╉',
        [Weight::Light, Weight::Double, Weight::None, Weight::None] => '╙',
        [Weight::Light, Weight::Double, Weight::None, Weight::Double] => '╟',
        [Weight::Light, Weight::Double, Weight::Light, Weight::None] => '╨',
        [Weight::Light, Weight::Double, Weight::Light, Weight::Double] => '╫',
        [Weight::Heavy, Weight::None, Weight::None, Weight::None] => '╺',
        [Weight::Heavy, Weight::None, Weight::None, Weight::Light] => '┍',
        [Weight::Heavy, Weight::None, Weight::None, Weight::Heavy] => '┏',
        [Weight::Heavy, Weight::None, Weight::Light, Weight::None] => '╼',
        [Weight::Heavy, Weight::None, Weight::Light, Weight::Light] => '┮',
        [Weight::Heavy, Weight::None, Weight::Light, Weight::Heavy] => '┲',
        [Weight::Heavy, Weight::None, Weight::Heavy, Weight::None] => '━',
        [Weight::Heavy, Weight::None, Weight::Heavy, Weight::Light] => '┯',
        [Weight::Heavy, Weight::None, Weight::Heavy, Weight::Heavy] => '┳',
        [Weight::Heavy, Weight::Light, Weight::None, Weight::None] => '┕',
        [Weight::Heavy, Weight::Light, Weight::None, Weight::Light] => '┝',
        [Weight::Heavy, Weight::Light, Weight::None, Weight::Heavy] => '┢',
        [Weight::Heavy, Weight::Light, Weight::Light, Weight::None] => '┶',
        [Weight::Heavy, Weight::Light, Weight::Light, Weight::Light] => '┾',
        [Weight::Heavy, Weight::Light, Weight::Light, Weight::Heavy] => '╆',
        [Weight::Heavy, Weight::Light, Weight::Heavy, Weight::None] => '┷',
        [Weight::Heavy, Weight::Light, Weight::Heavy, Weight::Light] => '┿',
        [Weight::Heavy, Weight::Light, Weight::Heavy, Weight::Heavy] => '╈',
        [Weight::Heavy, Weight::Heavy, Weight::None, Weight::None] => '┗',
        [Weight::Heavy, Weight::Heavy, Weight::None, Weight::Light] => '┡',
        [Weight::Heavy, Weight::Heavy, Weight::None, Weight::Heavy] => '┣',
        [Weight::Heavy, Weight::Heavy, Weight::Light, Weight::None] => '┺',
        [Weight::Heavy, Weight::Heavy, Weight::Light, Weight::Light] => '╄',
        [Weight::Heavy, Weight::Heavy, Weight::Light, Weight::Heavy] => '╊',
        [Weight::Heavy, Weight::Heavy, Weight::Heavy, Weight::None] => '┻',
        [Weight::Heavy, Weight::Heavy, Weight::Heavy, Weight::Light] => '╇',
        [Weight::Heavy, Weight::Heavy, Weight::Heavy, Weight::Heavy] => '╋',
        [Weight::Double, Weight::None, Weight::None, Weight::Light] => '╒',
        [Weight::Double, Weight::None, Weight::None, Weight::Double] => '╔',
        [Weight::Double, Weight::None, Weight::Double, Weight::None] => '═',
        [Weight::Double, Weight::None, Weight::Double, Weight::Light] => '╤',
        [Weight::Double, Weight::None, Weight::Double, Weight::Double] => '╦',
        [Weight::Double, Weight::Light, Weight::None, Weight::None] => '╘',
        [Weight::Double, Weight::Light, Weight::None, Weight::Light] => '╞',
        [Weight::Double, Weight::Light, Weight::Double, Weight::None] => '╧',
        [Weight::Double, Weight::Light, Weight::Double, Weight::Light] => '╪',
        [Weight::Double, Weight::Double, Weight::None, Weight::None] => '╚',
        [Weight::Double, Weight::Double, Weight::None, Weight::Double] => '╠',
        [Weight::Double, Weight::Double, Weight::Double, Weight::None] => '╩',
        [Weight::Double, Weight::Double, Weight::Double, Weight::Double] => '╬',

        other => panic!("no box-drawing character for weight combination {:?}", other),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Coord {
    pub(crate) x: usize,
    pub(crate) y: usize,
}

impl Coord {
    pub(crate) fn new(x: usize, y: usize) -> Coord {
        Coord { x, y }
    }

    pub(crate) fn shifted_by(self, offset: Coord) -> Coord {
        Coord::new(self.x + offset.x, self.y + offset.y)
    }
}

impl From<(usize, usize)> for Coord {
    fn from((x, y): (usize, usize)) -> Coord {
        Coord::new(x, y)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Size {
    pub(crate) width: usize,
    pub(crate) height: usize,
}

impl Size {
    pub(crate) fn new(width: usize, height: usize) -> Size {
        Size { width, height }
    }
}

/// A rectangular area of cells.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Rect {
    top_left: Coord,
    size: Size,
}

impl Rect {
    pub(crate) fn new(top_left: Coord, size: Size) -> Rect {
        Rect { top_left, size }
    }

    /// A one cell tall rectangle.
    pub(crate) fn row(start: Coord, length: usize) -> Rect {
        Rect::new(start, Size::new(length, 1))
    }

    /// The rectangle spanning both corners, each of which it covers.
    fn from_corners(top_left: Coord, bottom_right: Coord) -> Rect {
        Rect::new(
            top_left,
            Size::new(bottom_right.x - top_left.x + 1, bottom_right.y - top_left.y + 1),
        )
    }

    pub(crate) fn left(self) -> usize {
        self.top_left.x
    }

    pub(crate) fn top(self) -> usize {
        self.top_left.y
    }

    pub(crate) fn right(self) -> usize {
        assert!(self.size.width > 0, "Rect::right() called on an empty rectangle");
        self.top_left.x + self.size.width - 1
    }

    pub(crate) fn bottom(self) -> usize {
        assert!(self.size.height > 0, "Rect::bottom() called on an empty rectangle");
        self.top_left.y + self.size.height - 1
    }

    pub(crate) fn shifted_by(self, offset: Coord) -> Rect {
        Rect::new(self.top_left.shifted_by(offset), self.size)
    }

    /// The cells the rectangle encloses, excluding its own border.
    /// One too narrow or short to enclose anything gives an empty
    /// rectangle.
    fn interior(self) -> Rect {
        Rect::new(
            Coord::new(self.top_left.x + 1, self.top_left.y + 1),
            Size::new(self.size.width.saturating_sub(2), self.size.height.saturating_sub(2)),
        )
    }

    /// Whether the coordinate falls inside the rectangle. An empty one
    /// contains nothing.
    pub(crate) fn contains(self, coord: Coord) -> bool {
        coord.x >= self.left()
            && coord.x - self.left() < self.size.width
            && coord.y >= self.top()
            && coord.y - self.top() < self.size.height
    }

    /// Every cell the rectangle covers, row by row.
    pub(crate) fn coords(self) -> impl Iterator<Item = Coord> {
        let Coord { x: left, y: top } = self.top_left;
        let Size { width, height } = self.size;

        (top..top + height).flat_map(move |y| (left..left + width).map(move |x| Coord::new(x, y)))
    }
}

type Offset = (usize, usize);

/// Gap between the grid lines and the frame: FRAME_MARGIN_X columns on each
/// side, FRAME_MARGIN_Y rows above and below. Terminal character cells are
/// taller than they are wide, so the vertical gap grows at half the rate of
/// the horizontal one to look even.
const FRAME_MARGIN_X: usize = 1;
const FRAME_MARGIN_Y: usize = FRAME_MARGIN_X / 2;

/// Gap between the frame's own border and the canvas edge.
const CANVAS_MARGIN_X: usize = 0;
const CANVAS_MARGIN_Y: usize = 0;

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
pub(crate) fn tiles_at(position: impl Into<Coord>, dimensions: GridDimensions) -> Vec<TileCoord> {
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
fn top_left(tile_coord: TileCoord) -> Coord {
    tile_offset_to_coord(tile_coord, (0, 0))
}
fn top_mid(tile_coord: TileCoord) -> Coord {
    tile_offset_to_coord(tile_coord, (2, 0))
}
fn top_right(tile_coord: TileCoord) -> Coord {
    tile_offset_to_coord(tile_coord, (TILE_WIDTH - 1, 0))
}
fn left_side(tile_coord: TileCoord) -> Coord {
    tile_offset_to_coord(tile_coord, (0, 1))
}
fn center_left(tile_coord: TileCoord) -> Coord {
    tile_offset_to_coord(tile_coord, (1, 1))
}
fn center_mid(tile_coord: TileCoord) -> Coord {
    tile_offset_to_coord(tile_coord, (2, 1))
}
fn center_right(tile_coord: TileCoord) -> Coord {
    tile_offset_to_coord(tile_coord, (3, 1))
}
fn right_side(tile_coord: TileCoord) -> Coord {
    tile_offset_to_coord(tile_coord, (TILE_WIDTH - 1, 1))
}
fn bottom_left(tile_coord: TileCoord) -> Coord {
    tile_offset_to_coord(tile_coord, (0, TILE_HEIGHT - 1))
}
fn bottom_mid(tile_coord: TileCoord) -> Coord {
    tile_offset_to_coord(tile_coord, (2, TILE_HEIGHT - 1))
}
fn bottom_right(tile_coord: TileCoord) -> Coord {
    tile_offset_to_coord(tile_coord, (TILE_WIDTH - 1, TILE_HEIGHT - 1))
}

fn tile_rect(tile: TileCoord) -> Rect {
    Rect::from_corners(top_left(tile), bottom_right(tile))
}

/// The tile's three content coordinates.
fn tile_content_coords(tile: TileCoord) -> impl Iterator<Item = Coord> {
    tile_rect(tile).interior().coords()
}

/// The tile's entire footprint: all four of its borders and its content.
fn tile_full_coords(tile: TileCoord) -> impl Iterator<Item = Coord> {
    tile_rect(tile).coords()
}

/// The tile's top border, excluding its left and right corners.
fn top_border_middle(tile: TileCoord) -> impl Iterator<Item = Coord> {
    Rect::new(tile_offset_to_coord(tile, (1, 0)), Size::new(TILE_WIDTH - 2, 1)).coords()
}

/// The tile's bottom border, excluding its left and right corners.
fn bottom_border_middle(tile: TileCoord) -> impl Iterator<Item = Coord> {
    Rect::new(tile_offset_to_coord(tile, (1, TILE_HEIGHT - 1)), Size::new(TILE_WIDTH - 2, 1))
        .coords()
}

/// The board consists of the frame enclosing the grid and the status
/// bar on the row beneath it.
#[derive(Clone, Copy)]
struct Board {
    frame: Rect,
    status_bar_start: Coord,
}

impl Board {
    fn new((width, height): GridDimensions) -> Board {
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
    fn rect(self) -> Rect {
        let bottom_right = Coord::new(self.frame.right(), self.status_bar_start.y);
        Rect::from_corners(self.frame.top_left, bottom_right)
    }
}

/// One screen coordinate: either an accumulated `Code` segment (resolved to
/// a box-drawing character via `glyph()`), or a literal marker character
/// for content that isn't expressible as arm weights at all, like the
/// endpoint/source boxes.
#[derive(Clone, Copy)]
enum Cell {
    Segment(Code),
    Marker(char),
}

/// The ways a position can be marked.
#[derive(Clone, Copy)]
pub(crate) enum Mark {
    Bold,
    Dimmed,
    Reversed,
}

/// The marks a position carries.
#[derive(Clone, Copy, Default)]
struct Marks {
    /// Whatever has focus.
    bold: bool,
    /// A disabled button.
    dimmed: bool,
    reversed: bool,
}

/// A dense grid of `Cell`, one per screen coordinate, initialized to a
/// blank segment (`BLANK`). Sized from the board and the menu passed to
/// `new()`.
pub(crate) struct Canvas {
    cells: Vec<Vec<Cell>>,
    /// How each position renders, independent of `cells`' own content.
    marks: Vec<Vec<Marks>>,
}

impl Canvas {
    fn new(board_rect: Rect, menu_size: Size) -> Canvas {
        // The board and the menu sit end to end across the canvas, so
        // their widths add. They overlap down it, so the taller of the
        // two sets the height.
        let size = Size::new(
            board_rect.right() + 1 + menu_size.width + CANVAS_MARGIN_X,
            max(board_rect.bottom() + 1, board_rect.top() + menu_size.height) + CANVAS_MARGIN_Y,
        );

        Canvas {
            cells: vec![vec![Cell::Segment(BLANK); size.width]; size.height],
            marks: vec![vec![Marks::default(); size.width]; size.height],
        }
    }

    fn width(&self) -> usize {
        self.cells[0].len()
    }

    fn height(&self) -> usize {
        self.cells.len()
    }

    fn draw_code(&mut self, coord: impl Into<Coord>, code: Code) {
        let coord = coord.into();
        match &mut self.cells[coord.y][coord.x] {
            Cell::Segment(existing) => *existing = combine(*existing, code),
            Cell::Marker(_) => panic!("draw_code() called on a marked cell"),
        }
    }

    fn draw_char(&mut self, coord: impl Into<Coord>, c: char) {
        let coord = coord.into();
        self.cells[coord.y][coord.x] = Cell::Marker(c);
    }

    /// Draws each character of `text` in order, starting at `start` and
    /// advancing one column per character. Characters past the canvas's
    /// right edge are dropped.
    pub(crate) fn draw_text(&mut self, start: impl Into<Coord>, text: &str) {
        let start = start.into();
        for (offset_x, c) in text.chars().enumerate() {
            let x = start.x + offset_x;
            if x >= self.width() {
                break;
            }
            self.draw_char((x, start.y), c);
        }
    }

    fn char_at(&self, coord: impl Into<Coord>) -> char {
        let coord = coord.into();
        match self.cells[coord.y][coord.x] {
            Cell::Segment(code) => glyph(code),
            Cell::Marker(c) => c,
        }
    }

    fn mark(&mut self, coord: impl Into<Coord>, mark: Mark) {
        let coord = coord.into();
        let marks = &mut self.marks[coord.y][coord.x];

        match mark {
            Mark::Bold => marks.bold = true,
            Mark::Dimmed => marks.dimmed = true,
            Mark::Reversed => marks.reversed = true,
        }
    }

    pub(crate) fn mark_region(&mut self, coords: impl IntoIterator<Item = Coord>, mark: Mark) {
        for coord in coords {
            self.mark(coord, mark);
        }
    }

    /// Toggles reverse video at this position. Used only by the cursor,
    /// where landing on an already-reversed locked tile cancels back to plain.
    fn toggle_reversed(&mut self, coord: impl Into<Coord>) {
        let coord = coord.into();
        let marks = &mut self.marks[coord.y][coord.x];
        marks.reversed = !marks.reversed;
    }

    fn toggle_reversed_region(&mut self, coords: impl IntoIterator<Item = Coord>) {
        for coord in coords {
            self.toggle_reversed(coord);
        }
    }

    fn is_reversed(&self, coord: impl Into<Coord>) -> bool {
        let coord = coord.into();
        self.marks[coord.y][coord.x].reversed
    }

    /// Draws text, marking every cell it covers.
    pub(crate) fn draw_text_marked(
        &mut self,
        start: impl Into<Coord>,
        text: &str,
        mark: Option<Mark>,
    ) {
        let start = start.into();
        self.draw_text(start, text);

        let covered = Rect::row(start, text.chars().count());
        if let Some(mark) = mark {
            self.mark_region(covered.coords(), mark);
        }
    }

    /// How strongly a position renders. A terminal has one intensity to
    /// set, so where a position is both dim and bold, dim wins, since being
    /// unavailable outranks having focus.
    fn intensity_at(&self, coord: impl Into<Coord>) -> Intensity {
        let coord = coord.into();
        let marks = self.marks[coord.y][coord.x];

        if marks.dimmed {
            Intensity::Dim
        } else if marks.bold {
            Intensity::Bold
        } else {
            Intensity::Normal
        }
    }
}

/// Draws a line of the given weight from `start` to `end`, inclusive, along
/// whichever of column or row they share. The two ends don't need to be
/// given in order. Each endpoint only gets the arm pointing back into
/// the line, not the one pointing past it. Panics if `start` and `end` are
/// neither on the same row nor the same column.
pub(crate) fn draw_line(
    canvas: &mut Canvas,
    start: impl Into<Coord>,
    end: impl Into<Coord>,
    weight: Weight,
) {
    let (mut start, mut end) = (start.into(), end.into());
    if start == end {
        return;
    }

    if start.x > end.x {
        std::mem::swap(&mut start.x, &mut end.x);
    }
    if start.y > end.y {
        std::mem::swap(&mut start.y, &mut end.y);
    }

    // The line starts and ends in the middle of a cell, so each endpoint stops short of a full arm.
    if start.y == end.y {
        for x in start.x..=end.x {
            let right = if x == end.x { Weight::None } else { weight };
            let left = if x == start.x { Weight::None } else { weight };
            let code = [right, Weight::None, left, Weight::None];
            canvas.draw_code((x, start.y), code);
        }
    } else if start.x == end.x {
        for y in start.y..=end.y {
            let up = if y == start.y { Weight::None } else { weight };
            let down = if y == end.y { Weight::None } else { weight };
            let code = [Weight::None, up, Weight::None, down];
            canvas.draw_code((start.x, y), code);
        }
    } else {
        panic!("draw_line only supports horizontal or vertical lines");
    }
}

/// Draws the fixed tile borders between cells, independent of any wire.
fn draw_grid_lines(canvas: &mut Canvas, (width, height): GridDimensions) {
    for tile_y in 0..=height {
        draw_line(canvas, top_left((0, tile_y)), top_left((width, tile_y)), Weight::Light);
    }

    for tile_x in 0..=width {
        draw_line(canvas, top_left((tile_x, 0)), top_left((tile_x, height)), Weight::Light);
    }
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

/// The outline runs through the centers of `Rect`'s outermost cells.
pub(crate) fn draw_rect_outline(canvas: &mut Canvas, rect: Rect, weight: Weight) {
    let (left, top, right, bottom) = (rect.left(), rect.top(), rect.right(), rect.bottom());

    draw_line(canvas, (left, top), (right, top), weight);
    draw_line(canvas, (left, bottom), (right, bottom), weight);
    draw_line(canvas, (left, top), (left, bottom), weight);
    draw_line(canvas, (right, top), (right, bottom), weight);
}

const FRAME_WEIGHT: Weight = Weight::Double;

/// Draws the outer presentation frame, offset from the grid lines by
/// FRAME_MARGIN_X/FRAME_MARGIN_Y and from the canvas edge by
/// CANVAS_MARGIN_X/CANVAS_MARGIN_Y.
fn draw_frame(canvas: &mut Canvas, frame: Rect) {
    draw_rect_outline(canvas, frame, FRAME_WEIGHT);
}

fn draw_status_bar(canvas: &mut Canvas, start: Coord, status: &str) {
    canvas.draw_text(start, status);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Intensity {
    Normal,
    Bold,
    Dim,
}

impl Intensity {
    fn as_attribute(self) -> Attribute {
        match self {
            Intensity::Normal => Attribute::NormalIntensity,
            Intensity::Bold => Attribute::Bold,
            Intensity::Dim => Attribute::Dim,
        }
    }
}

/// Flattens the drawn canvas into one line of text per screen row.
/// Lines are cut at the terminal's width here, where the escape codes
/// are written, so a cut line still ends with its resets.
fn flatten_to_lines(canvas: &Canvas, terminal_columns: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::with_capacity(canvas.height());
    for y in 0..canvas.height() {
        let mut line = String::with_capacity(canvas.width());
        let mut reversed = false;
        let mut active_intensity = Intensity::Normal;

        for x in 0..canvas.width().min(terminal_columns) {
            if canvas.is_reversed((x, y)) != reversed {
                reversed = !reversed;
                let attribute = if reversed {
                    Attribute::Reverse
                } else {
                    Attribute::NoReverse
                };
                line.push_str(&SetAttribute(attribute).to_string());
            }

            let cell_intensity = canvas.intensity_at((x, y));
            if cell_intensity != active_intensity {
                active_intensity = cell_intensity;
                line.push_str(&SetAttribute(active_intensity.as_attribute()).to_string());
            }

            line.push(canvas.char_at((x, y)));
        }

        if reversed {
            line.push_str(&SetAttribute(Attribute::NoReverse).to_string());
        }
        if active_intensity != Intensity::Normal {
            line.push_str(&SetAttribute(Attribute::NormalIntensity).to_string());
        }

        lines.push(line);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::ActionAvailability;
    use crate::net::TileCoordNeighbors;

    /// Every action available, so nothing is dimmed.
    fn nothing_dimmed() -> ActionAvailability {
        ActionAvailability { can_undo: true, can_redo: true }
    }

    /// A canvas sized for the board and its menu.
    fn canvas(dimensions: GridDimensions) -> Canvas {
        Canvas::new(Board::new(dimensions).rect(), Menu::new().size())
    }

    #[test]
    fn blank_is_space() {
        assert_eq!(glyph(BLANK), ' ');
    }

    #[test]
    fn four_way_light_cross() {
        assert_eq!(glyph([Weight::Light; 4]), '┼');
    }

    #[test]
    fn double_corner() {
        // right=double, up=none, left=none, down=double -> top-left
        // corner of a double-line box.
        assert_eq!(glyph([Weight::Double, Weight::None, Weight::None, Weight::Double]), '╔');
    }

    #[test]
    fn combine_takes_elementwise_max() {
        let a = [Weight::None, Weight::Light, Weight::None, Weight::Light];
        let b = [Weight::Heavy, Weight::None, Weight::Heavy, Weight::None];
        assert_eq!(combine(a, b), [Weight::Heavy, Weight::Light, Weight::Heavy, Weight::Light]);
    }

    #[test]
    #[should_panic(expected = "no box-drawing character for weight combination")]
    fn heavy_and_double_has_no_glyph() {
        glyph([Weight::Heavy, Weight::Double, Weight::Heavy, Weight::Double]);
    }

    #[test]
    fn draw_line_normalizes_reversed_endpoints() {
        let start = (0, 0);
        let end = (3, 0);

        let mut natural = canvas((2, 2));
        draw_line(&mut natural, start, end, Weight::Light);

        let mut reversed = canvas((2, 2));
        draw_line(&mut reversed, end, start, Weight::Light);

        for y in 0..natural.height() {
            for x in 0..natural.width() {
                let coord = Coord::new(x, y);
                assert_eq!(natural.char_at(coord), reversed.char_at(coord));
            }
        }
    }

    #[test]
    #[should_panic(expected = "draw_line only supports horizontal or vertical lines")]
    fn draw_line_panics_on_diagonal() {
        let mut canvas = canvas((2, 2));
        draw_line(&mut canvas, (0, 0), (3, 3), Weight::Light);
    }

    #[test]
    fn draw_line_start_equals_end_is_noop() {
        let mut canvas = canvas((2, 2));
        draw_line(&mut canvas, (1, 1), (1, 1), Weight::Light);
        assert_eq!(canvas.char_at((1, 1)), ' ');
    }

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

    /// A 5x5 board's frame is 25 columns wide and the menu adds 60.
    /// The board is the taller of the two, so it sets the height.
    #[test]
    fn canvas_new_computes_expected_size() {
        let canvas = canvas((5, 5));
        assert_eq!(canvas.width(), 85);
        assert_eq!(canvas.height(), 14);
    }

    #[test]
    #[should_panic(expected = "Board::new() called with a zero-tile board")]
    fn board_new_panics_on_zero_tile_board() {
        Board::new((0, 0));
    }

    #[test]
    #[should_panic(expected = "draw_code() called on a marked cell")]
    fn draw_code_panics_on_marked_cell() {
        let mut canvas = canvas((1, 1));
        canvas.draw_char((0, 0), 'x');
        canvas.draw_code((0, 0), BLANK);
    }

    #[test]
    fn render_game_does_not_panic_across_many_generated_boards() {
        for _ in 0..100 {
            let session = crate::net::Session::new();
            let puzzle = session.puzzle();
            render_game(puzzle, Styles::default(), &mut Menu::new(), nothing_dimmed(), usize::MAX);
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

    #[test]
    fn flatten_wraps_reversed_cells_in_escape_codes() {
        let mut canvas = canvas((1, 1));
        canvas.mark((0, 0), Mark::Reversed);

        let lines = flatten_to_lines(&canvas, usize::MAX);
        assert!(lines[0].starts_with(&SetAttribute(Attribute::Reverse).to_string()));
        assert!(lines[0].contains(&SetAttribute(Attribute::NoReverse).to_string()));
    }

    #[test]
    fn flatten_wraps_dimmed_cells_in_escape_codes() {
        let mut canvas = canvas((1, 1));
        canvas.mark_region([Coord::new(0, 0)], Mark::Dimmed);

        let lines = flatten_to_lines(&canvas, usize::MAX);
        assert!(lines[0].starts_with(&SetAttribute(Attribute::Dim).to_string()));
        assert!(lines[0].contains(&SetAttribute(Attribute::NormalIntensity).to_string()));
    }

    /// A placed menu draws where it was placed. Beside a 5x5 board, whose
    /// frame ends at column 24, the divider sits at 26 and the tabs
    /// start at 28.
    #[test]
    fn menu_draws_where_it_is_placed() {
        let mut menu = Menu::new();
        // Menu Controls' header, opened so a legend is drawn too.
        menu.click((3, 5));

        let board_rect = Board::new((5, 5)).rect();
        let mut canvas = Canvas::new(board_rect, menu.size());
        menu.set_placement(board_rect, usize::MAX);
        menu.draw(&mut canvas, nothing_dimmed(), Styles::default());

        // The divider, ending on the board's last row, then the first
        // button's corner, Cursor Style's header line and the legend's
        // first label.
        assert_eq!(canvas.char_at((26, 5)), '│');
        assert_eq!(canvas.char_at((26, 13)), '╵');
        assert_eq!(canvas.char_at((26, 14)), ' ');
        assert_eq!(canvas.char_at((28, 0)), '┌');
        assert_eq!(canvas.char_at((29, 3)), '─');
        assert_eq!(canvas.char_at((28, 6)), 'G');
    }

    /// A header's line and arrow stop at the terminal's edge when the
    /// menu runs past it.
    #[test]
    fn headers_stop_at_the_terminal_edge() {
        let mut menu = Menu::new();
        let board_rect = Board::new((5, 5)).rect();
        let mut canvas = Canvas::new(board_rect, menu.size());

        // Cursor Style's header would run to column 84 in a terminal
        // wide enough, but this one is 60 columns wide.
        menu.set_placement(board_rect, 60);
        menu.draw(&mut canvas, nothing_dimmed(), Styles::default());

        assert_eq!(canvas.char_at((59, 3)), '▼');
        assert_eq!(canvas.char_at((60, 3)), ' ');
    }

    /// An unavailable action's button is dimmed, so a rendered game
    /// carries the dim attribute only when one of them is unavailable.
    #[test]
    fn render_game_dims_unavailable_menu_buttons() {
        let session = crate::net::Session::new();
        let puzzle = session.puzzle();
        let dim = SetAttribute(Attribute::Dim).to_string();
        let mut menu = Menu::new();
        let mut render = |availability| {
            render_game(puzzle, Styles::default(), &mut menu, availability, usize::MAX)
        };

        let fresh = ActionAvailability { can_undo: false, can_redo: false };
        let mid_game = ActionAvailability { can_undo: true, can_redo: true };

        assert!(render(fresh).contains(&dim));
        assert!(!render(mid_game).contains(&dim));
    }

    #[test]
    fn marking_reversed_twice_stays_reversed() {
        let mut canvas = canvas((1, 1));
        canvas.mark((0, 0), Mark::Reversed);
        canvas.mark((0, 0), Mark::Reversed);

        assert!(canvas.is_reversed((0, 0)));
    }

    #[test]
    fn toggle_reversed_twice_cancels_out() {
        let mut canvas = canvas((1, 1));
        canvas.toggle_reversed((0, 0));
        canvas.toggle_reversed((0, 0));

        assert!(!canvas.is_reversed((0, 0)));
    }

    #[test]
    fn toggle_reversed_cancels_a_prior_reversed_mark() {
        let mut canvas = canvas((1, 1));
        canvas.mark((0, 0), Mark::Reversed);
        canvas.toggle_reversed((0, 0));

        assert!(!canvas.is_reversed((0, 0)));
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

    /// A line cut at the terminal's edge still ends with its resets, so
    /// reverse video can't run on past it.
    #[test]
    fn flatten_cuts_lines_at_the_terminal_edge_and_still_closes_them() {
        let mut canvas = canvas((1, 1));
        canvas.mark_region([Coord::new(0, 0), Coord::new(1, 0)], Mark::Reversed);

        let lines = flatten_to_lines(&canvas, 1);

        let reverse = SetAttribute(Attribute::Reverse);
        let no_reverse = SetAttribute(Attribute::NoReverse);
        assert_eq!(lines[0], format!("{reverse}{}{no_reverse}", canvas.char_at((0, 0))));
    }
}

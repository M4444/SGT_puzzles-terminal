//! Renders a Net board as a box-drawing string for the terminal.
//!
//! `render_board()` draws a given `NetPuzzle` onto a `Canvas` in phases
//! (grid lines, wires and endpoints, source, barriers, cursor, frame),
//! then flattens the result to text.

use crate::net::{Cursor, GridDimensions, NetPuzzle, TileCoord, Tiles};
use crossterm::style::{Attribute, SetAttribute};
use std::cmp::max;

pub fn render_board(
    puzzle: &NetPuzzle,
    cursor_style: CursorStyle,
    lock_style: LockStyle,
) -> String {
    let dimensions = puzzle.dimensions;
    let source = source_position(puzzle);

    let mut canvas = Canvas::new(dimensions);

    draw_grid_lines(&mut canvas, dimensions);
    draw_wires_and_endpoints(&mut canvas, &puzzle.tiles, dimensions);
    draw_source(&mut canvas, source);
    draw_barriers(&mut canvas, dimensions, puzzle.wrapping);
    draw_locked(&mut canvas, &puzzle.tiles, dimensions, lock_style);
    draw_cursor(&mut canvas, puzzle.cursor, cursor_style);
    draw_frame(&mut canvas);

    flatten_to_lines(&canvas).join("\n")
}

/// How strongly a line is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Weight {
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
    [
        max(a[0], b[0]),
        max(a[1], b[1]),
        max(a[2], b[2]),
        max(a[3], b[3]),
    ]
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

/// Where the source tile is. Always the board's center; doesn't reflect
/// moves made during play.
fn source_position(puzzle: &NetPuzzle) -> TileCoord {
    let (width, height) = puzzle.dimensions;
    (width / 2, height / 2)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Coord {
    x: usize,
    y: usize,
}

impl Coord {
    fn new(x: usize, y: usize) -> Coord {
        Coord { x, y }
    }
}

impl From<(usize, usize)> for Coord {
    fn from((x, y): (usize, usize)) -> Coord {
        Coord::new(x, y)
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

/// Addressing a tile's contents can be done relative to the tile: the tile
/// is given as `(tile_x, tile_y)`, and the offset within it as a
/// quarter-step `offset_x` across (0..TILE_WIDTH) and a half-step
/// `offset_y` down (0..TILE_HEIGHT), overlapping by one step with each
/// neighbouring tile to merge borders between them.
fn tile_offset_to_coord((tile_x, tile_y): TileCoord, (offset_x, offset_y): Offset) -> Coord {
    let x = GRID_OFFSET_X + (TILE_WIDTH - 1) * tile_x + offset_x;
    let y = GRID_OFFSET_Y + (TILE_HEIGHT - 1) * tile_y + offset_y;
    Coord::new(x, y)
}

// Helper functions name specific coordinates within a tile's footprint:
//
//               top_mid    a = center_left
//                  │       b = center_mid
//                  ▼       c = center_right
// top_left ─────►┌───┐◄─── top_right
// left_side ────►│abc│
// bottom_left ──►└───┘◄─── bottom_right
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
fn bottom_left(tile_coord: TileCoord) -> Coord {
    tile_offset_to_coord(tile_coord, (0, TILE_HEIGHT - 1))
}
fn bottom_right(tile_coord: TileCoord) -> Coord {
    tile_offset_to_coord(tile_coord, (TILE_WIDTH - 1, TILE_HEIGHT - 1))
}

/// The tile's three content coordinates.
fn tile_content_coords(tile: TileCoord) -> [Coord; 3] {
    [center_left(tile), center_mid(tile), center_right(tile)]
}

/// The tile's entire footprint: all four of its borders and its content.
fn tile_full_coords(tile: TileCoord) -> Vec<Coord> {
    let mut coords = Vec::with_capacity(TILE_WIDTH * TILE_HEIGHT);
    for offset_y in 0..TILE_HEIGHT {
        for offset_x in 0..TILE_WIDTH {
            coords.push(tile_offset_to_coord(tile, (offset_x, offset_y)));
        }
    }
    coords
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

/// A dense grid of `Cell`, one per screen coordinate, initialized to a
/// blank segment (`BLANK`). Sized from the tile-grid dimensions passed to
/// `new()`, not any fixed constant. Also stores which positions render in
/// reverse video, and the frame's bounds.
struct Canvas {
    cells: Vec<Vec<Cell>>,
    /// Which positions render in reverse video, independent of `cells`'
    /// own content.
    reversed: Vec<Vec<bool>>,
    frame_left: usize,
    frame_top: usize,
    frame_right: usize,
    frame_bottom: usize,
}

impl Canvas {
    fn new((tile_width, tile_height): GridDimensions) -> Canvas {
        let board_width = (TILE_WIDTH - 1) * tile_width + 1;
        let board_height = (TILE_HEIGHT - 1) * tile_height + 1;
        let canvas_width = board_width + 2 * FRAME_MARGIN_X + 2 + 2 * CANVAS_MARGIN_X;
        let canvas_height = board_height + 2 * FRAME_MARGIN_Y + 2 + 2 * CANVAS_MARGIN_Y;
        Canvas {
            cells: vec![vec![Cell::Segment(BLANK); canvas_width]; canvas_height],
            reversed: vec![vec![false; canvas_width]; canvas_height],
            frame_left: CANVAS_MARGIN_X,
            frame_top: CANVAS_MARGIN_Y,
            frame_right: canvas_width - 1 - CANVAS_MARGIN_X,
            frame_bottom: canvas_height - 1 - CANVAS_MARGIN_Y,
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

    fn char_at(&self, coord: impl Into<Coord>) -> char {
        let coord = coord.into();
        match self.cells[coord.y][coord.x] {
            Cell::Segment(code) => glyph(code),
            Cell::Marker(c) => c,
        }
    }

    fn mark_reversed(&mut self, coord: impl Into<Coord>) {
        let coord = coord.into();
        self.reversed[coord.y][coord.x] = true;
    }

    fn is_reversed(&self, coord: impl Into<Coord>) -> bool {
        let coord = coord.into();
        self.reversed[coord.y][coord.x]
    }
}

/// Draws a line of the given weight from `start` to `end`, inclusive, along
/// whichever of column or row they share. `start` and `end` don't need to
/// be given in order. Each endpoint only gets the arm pointing back into
/// the line, not the one pointing past it. Panics if `start` and `end` are
/// neither on the same row nor the same column.
fn draw_line(canvas: &mut Canvas, start: impl Into<Coord>, end: impl Into<Coord>, weight: Weight) {
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
fn draw_wires_and_endpoints(canvas: &mut Canvas, tiles: &Tiles, (width, height): GridDimensions) {
    for tile_y in 0..height {
        for tile_x in 0..width {
            let tile = tiles[tile_y][tile_x];
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
                draw_line(canvas, center, left_side((tile_x + 1, tile_y)), weight);
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
                draw_line(canvas, center, top_mid((tile_x, tile_y + 1)), weight);
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

/// How the keyboard cursor's tile is visually marked.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CursorStyle {
    /// A heavy-weight box drawn around the tile's own border.
    Outline,
    /// Reverse video over just the tile's content cells.
    ReverseTileCenter,
    /// Reverse video over the entire tile: all four of its borders and
    /// its content.
    #[default]
    ReverseTileFull,
}

/// Highlights the keyboard cursor's tile, if it's currently shown.
fn draw_cursor(canvas: &mut Canvas, cursor: Cursor, style: CursorStyle) {
    if !cursor.visible {
        return;
    }
    match style {
        CursorStyle::Outline => {
            let tile = cursor.position;
            draw_line(canvas, top_left(tile), top_right(tile), Weight::Heavy);
            draw_line(canvas, bottom_left(tile), bottom_right(tile), Weight::Heavy);
            draw_line(canvas, top_left(tile), bottom_left(tile), Weight::Heavy);
            draw_line(canvas, top_right(tile), bottom_right(tile), Weight::Heavy);
        }
        CursorStyle::ReverseTileCenter => {
            canvas.mark_reversed(center_left(cursor.position));
            canvas.mark_reversed(center_mid(cursor.position));
            canvas.mark_reversed(center_right(cursor.position));
        }
        CursorStyle::ReverseTileFull => {
            for offset_y in 0..TILE_HEIGHT {
                for offset_x in 0..TILE_WIDTH {
                    canvas
                        .mark_reversed(tile_offset_to_coord(cursor.position, (offset_x, offset_y)));
                }
            }
        }
    }
}

/// How a locked tile is visually marked.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LockStyle {
    /// Reverse video over just the tile's content cells.
    ReverseTileCenter,
    /// Reverse video over the entire tile: all four of its borders and
    /// its content.
    #[default]
    ReverseTileFull,
}

/// Highlights every locked tile.
fn draw_locked(
    canvas: &mut Canvas,
    tiles: &Tiles,
    (width, height): GridDimensions,
    style: LockStyle,
) {
    for tile_y in 0..height {
        for tile_x in 0..width {
            if !tiles[tile_y][tile_x].locked {
                continue;
            }
            let coords = match style {
                LockStyle::ReverseTileCenter => tile_content_coords((tile_x, tile_y)).to_vec(),
                LockStyle::ReverseTileFull => tile_full_coords((tile_x, tile_y)),
            };
            for coord in coords {
                canvas.mark_reversed(coord);
            }
        }
    }
}

/// Draws barrier walls, fixed obstacles blocking a wire connection, in
/// heavy weight across the border they occupy. A wrapping grid has no
/// outer boundary, so this draws nothing; otherwise it forms the full
/// outer boundary ring.
fn draw_barriers(canvas: &mut Canvas, (width, height): GridDimensions, wrapping: bool) {
    if wrapping {
        return;
    }
    draw_line(canvas, top_left((0, 0)), top_left((width, 0)), Weight::Heavy);
    draw_line(canvas, top_left((0, height)), top_left((width, height)), Weight::Heavy);
    draw_line(canvas, top_left((0, 0)), top_left((0, height)), Weight::Heavy);
    draw_line(canvas, top_left((width, 0)), top_left((width, height)), Weight::Heavy);
}

const FRAME_WEIGHT: Weight = Weight::Double;

/// Draws the outer presentation frame, offset from the grid lines by
/// FRAME_MARGIN_X/FRAME_MARGIN_Y and from the canvas edge by
/// CANVAS_MARGIN_X/CANVAS_MARGIN_Y.
fn draw_frame(canvas: &mut Canvas) {
    let left = canvas.frame_left;
    let top = canvas.frame_top;
    let right = canvas.frame_right;
    let bottom = canvas.frame_bottom;
    draw_line(canvas, (left, top), (right, top), FRAME_WEIGHT);
    draw_line(canvas, (left, bottom), (right, bottom), FRAME_WEIGHT);
    draw_line(canvas, (left, top), (left, bottom), FRAME_WEIGHT);
    draw_line(canvas, (right, top), (right, bottom), FRAME_WEIGHT);
}

/// Flattens the drawn canvas into one line of text per screen row.
fn flatten_to_lines(canvas: &Canvas) -> Vec<String> {
    let mut lines: Vec<String> = Vec::with_capacity(canvas.height());
    for y in 0..canvas.height() {
        let mut line = String::with_capacity(canvas.width());
        let mut reversed = false;
        for x in 0..canvas.width() {
            if canvas.is_reversed((x, y)) != reversed {
                reversed = !reversed;
                let attribute = if reversed {
                    Attribute::Reverse
                } else {
                    Attribute::NoReverse
                };
                line.push_str(&SetAttribute(attribute).to_string());
            }
            line.push(canvas.char_at((x, y)));
        }
        if reversed {
            line.push_str(&SetAttribute(Attribute::NoReverse).to_string());
        }
        lines.push(line);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let mut natural = Canvas::new((2, 2));
        draw_line(&mut natural, start, end, Weight::Light);

        let mut reversed = Canvas::new((2, 2));
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
        let mut canvas = Canvas::new((2, 2));
        draw_line(&mut canvas, (0, 0), (3, 3), Weight::Light);
    }

    #[test]
    fn draw_line_start_equals_end_is_noop() {
        let mut canvas = Canvas::new((2, 2));
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
        assert_tile_boundary_shared((0, 0), (1, 0), (TILE_WIDTH - 1, 0));
    }

    #[test]
    fn tile_boundary_shared_y() {
        assert_tile_boundary_shared((0, 0), (0, 1), (0, TILE_HEIGHT - 1));
    }

    #[test]
    fn canvas_new_computes_expected_size() {
        let canvas = Canvas::new((5, 5));
        assert_eq!(canvas.width(), 25);
        assert_eq!(canvas.height(), 13);
    }

    #[test]
    #[should_panic(expected = "draw_code() called on a marked cell")]
    fn draw_code_panics_on_marked_cell() {
        let mut canvas = Canvas::new((1, 1));
        canvas.draw_char((0, 0), 'x');
        canvas.draw_code((0, 0), BLANK);
    }

    #[test]
    fn render_board_is_a_well_formed_rectangle() {
        let puzzle = crate::net::generate();
        let board = render_board(&puzzle, CursorStyle::Outline, LockStyle::default());
        let lines: Vec<&str> = board.lines().collect();
        assert_eq!(lines.len(), 13);
        for line in &lines {
            assert_eq!(line.chars().count(), 25);
        }
    }

    #[test]
    fn render_board_does_not_panic_across_many_generated_boards() {
        for _ in 0..100 {
            let puzzle = crate::net::generate();
            render_board(&puzzle, CursorStyle::Outline, LockStyle::default());
        }
    }

    #[test]
    fn draw_cursor_outline_boxes_the_tile() {
        let mut canvas = Canvas::new((3, 3));
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
        let mut canvas = Canvas::new((3, 3));
        let cursor = Cursor { position: (1, 1), visible: true };
        draw_cursor(&mut canvas, cursor, CursorStyle::ReverseTileCenter);

        assert!(canvas.is_reversed(center_left((1, 1))));
        assert!(canvas.is_reversed(center_mid((1, 1))));
        assert!(canvas.is_reversed(center_right((1, 1))));
        assert!(!canvas.is_reversed(top_left((1, 1))));
    }

    #[test]
    fn draw_cursor_reverse_tile_full_includes_shared_edge() {
        let mut canvas = Canvas::new((3, 3));
        let cursor = Cursor { position: (1, 1), visible: true };
        draw_cursor(&mut canvas, cursor, CursorStyle::ReverseTileFull);

        assert!(canvas.is_reversed(top_left((1, 1))));
        assert!(canvas.is_reversed(center_mid((1, 1))));
        assert!(canvas.is_reversed(top_left((2, 1))));
        assert!(canvas.is_reversed(top_left((1, 2))));
    }

    #[test]
    fn flatten_wraps_reversed_cells_in_escape_codes() {
        let mut canvas = Canvas::new((1, 1));
        canvas.mark_reversed((0, 0));

        let lines = flatten_to_lines(&canvas);
        assert!(lines[0].starts_with(&SetAttribute(Attribute::Reverse).to_string()));
        assert!(lines[0].contains(&SetAttribute(Attribute::NoReverse).to_string()));
    }

    #[test]
    fn draw_cursor_draws_nothing_when_not_visible() {
        let mut canvas = Canvas::new((3, 3));
        let cursor = Cursor { position: (1, 1), visible: false };
        draw_cursor(&mut canvas, cursor, CursorStyle::default());

        assert!(!canvas.is_reversed(top_left((1, 1))));
        assert!(!canvas.is_reversed(center_mid((1, 1))));
    }
}

//! Renders a Net board as a box-drawing string for the terminal.
//!
//! `render_board()` draws a board onto a `Canvas` in phases (grid lines,
//! wires and endpoints, source, barriers, frame), then flattens the result
//! to text.

use std::cmp::max;

pub fn render_board() -> String {
    let (grid, powered_tiles, source, dimensions, wrapping) = build_grid();

    let mut canvas = Canvas::new(dimensions);

    draw_grid_lines(&mut canvas, dimensions);
    draw_wires_and_endpoints(&mut canvas, &grid, &powered_tiles, dimensions);
    draw_source(&mut canvas, source);
    draw_barriers(&mut canvas, dimensions, wrapping);
    draw_frame(&mut canvas);

    flatten_to_lines(&canvas).join("\n")
}

const NONE: u8 = 0;
const LIGHT: u8 = 1;
const HEAVY: u8 = 2;
const DOUBLE: u8 = 3;

/// Per-arm line weight, order (right, up, left, down).
type Code = [u8; 4];

const BLANK: Code = [NONE, NONE, NONE, NONE];

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
        [0, 0, 0, 0] => ' ',
        [0, 0, 0, 1] => '╷',
        [0, 0, 0, 2] => '╻',
        [0, 0, 1, 0] => '╴',
        [0, 0, 1, 1] => '┐',
        [0, 0, 1, 2] => '┒',
        [0, 0, 1, 3] => '╖',
        [0, 0, 2, 0] => '╸',
        [0, 0, 2, 1] => '┑',
        [0, 0, 2, 2] => '┓',
        [0, 0, 3, 1] => '╕',
        [0, 0, 3, 3] => '╗',
        [0, 1, 0, 0] => '╵',
        [0, 1, 0, 1] => '│',
        [0, 1, 0, 2] => '╽',
        [0, 1, 1, 0] => '┘',
        [0, 1, 1, 1] => '┤',
        [0, 1, 1, 2] => '┧',
        [0, 1, 2, 0] => '┙',
        [0, 1, 2, 1] => '┥',
        [0, 1, 2, 2] => '┪',
        [0, 1, 3, 0] => '╛',
        [0, 1, 3, 1] => '╡',
        [0, 2, 0, 0] => '╹',
        [0, 2, 0, 1] => '╿',
        [0, 2, 0, 2] => '┃',
        [0, 2, 1, 0] => '┚',
        [0, 2, 1, 1] => '┦',
        [0, 2, 1, 2] => '┨',
        [0, 2, 2, 0] => '┛',
        [0, 2, 2, 1] => '┩',
        [0, 2, 2, 2] => '┫',
        [0, 3, 0, 3] => '║',
        [0, 3, 1, 0] => '╜',
        [0, 3, 1, 3] => '╢',
        [0, 3, 3, 0] => '╝',
        [0, 3, 3, 3] => '╣',
        [1, 0, 0, 0] => '╶',
        [1, 0, 0, 1] => '┌',
        [1, 0, 0, 2] => '┎',
        [1, 0, 0, 3] => '╓',
        [1, 0, 1, 0] => '─',
        [1, 0, 1, 1] => '┬',
        [1, 0, 1, 2] => '┰',
        [1, 0, 1, 3] => '╥',
        [1, 0, 2, 0] => '╾',
        [1, 0, 2, 1] => '┭',
        [1, 0, 2, 2] => '┱',
        [1, 1, 0, 0] => '└',
        [1, 1, 0, 1] => '├',
        [1, 1, 0, 2] => '┟',
        [1, 1, 1, 0] => '┴',
        [1, 1, 1, 1] => '┼',
        [1, 1, 1, 2] => '╁',
        [1, 1, 2, 0] => '┵',
        [1, 1, 2, 1] => '┽',
        [1, 1, 2, 2] => '╅',
        [1, 2, 0, 0] => '┖',
        [1, 2, 0, 1] => '┞',
        [1, 2, 0, 2] => '┠',
        [1, 2, 1, 0] => '┸',
        [1, 2, 1, 1] => '╀',
        [1, 2, 1, 2] => '╂',
        [1, 2, 2, 0] => '┹',
        [1, 2, 2, 1] => '╃',
        [1, 2, 2, 2] => '╉',
        [1, 3, 0, 0] => '╙',
        [1, 3, 0, 3] => '╟',
        [1, 3, 1, 0] => '╨',
        [1, 3, 1, 3] => '╫',
        [2, 0, 0, 0] => '╺',
        [2, 0, 0, 1] => '┍',
        [2, 0, 0, 2] => '┏',
        [2, 0, 1, 0] => '╼',
        [2, 0, 1, 1] => '┮',
        [2, 0, 1, 2] => '┲',
        [2, 0, 2, 0] => '━',
        [2, 0, 2, 1] => '┯',
        [2, 0, 2, 2] => '┳',
        [2, 1, 0, 0] => '┕',
        [2, 1, 0, 1] => '┝',
        [2, 1, 0, 2] => '┢',
        [2, 1, 1, 0] => '┶',
        [2, 1, 1, 1] => '┾',
        [2, 1, 1, 2] => '╆',
        [2, 1, 2, 0] => '┷',
        [2, 1, 2, 1] => '┿',
        [2, 1, 2, 2] => '╈',
        [2, 2, 0, 0] => '┗',
        [2, 2, 0, 1] => '┡',
        [2, 2, 0, 2] => '┣',
        [2, 2, 1, 0] => '┺',
        [2, 2, 1, 1] => '╄',
        [2, 2, 1, 2] => '╊',
        [2, 2, 2, 0] => '┻',
        [2, 2, 2, 1] => '╇',
        [2, 2, 2, 2] => '╋',
        [3, 0, 0, 1] => '╒',
        [3, 0, 0, 3] => '╔',
        [3, 0, 3, 0] => '═',
        [3, 0, 3, 1] => '╤',
        [3, 0, 3, 3] => '╦',
        [3, 1, 0, 0] => '╘',
        [3, 1, 0, 1] => '╞',
        [3, 1, 3, 0] => '╧',
        [3, 1, 3, 1] => '╪',
        [3, 3, 0, 0] => '╚',
        [3, 3, 0, 3] => '╠',
        [3, 3, 3, 0] => '╩',
        [3, 3, 3, 3] => '╬',

        // There's no Unicode glyph that mixes DOUBLE and HEAVY.
        // For those cases we will just use LIGHT instead of HEAVY.
        [0, 0, 2, 3] => '╖',
        [0, 0, 3, 2] => '╕',
        [0, 1, 3, 2] => '╡',
        [0, 2, 3, 0] => '╛',
        [0, 2, 3, 1] => '╡',
        [0, 2, 3, 2] => '╡',
        [0, 3, 2, 0] => '╜',
        [0, 3, 2, 3] => '╢',
        [1, 0, 2, 3] => '╥',
        [1, 3, 2, 0] => '╨',
        [1, 3, 2, 3] => '╫',
        [2, 0, 0, 3] => '╓',
        [2, 0, 1, 3] => '╥',
        [2, 0, 2, 3] => '╥',
        [2, 3, 0, 0] => '╙',
        [2, 3, 0, 3] => '╟',
        [2, 3, 1, 0] => '╨',
        [2, 3, 1, 3] => '╫',
        [2, 3, 2, 0] => '╨',
        [2, 3, 2, 3] => '╫',
        [3, 0, 0, 2] => '╒',
        [3, 0, 3, 2] => '╤',
        [3, 1, 0, 2] => '╞',
        [3, 1, 3, 2] => '╪',
        [3, 2, 0, 0] => '╘',
        [3, 2, 0, 1] => '╞',
        [3, 2, 0, 2] => '╞',
        [3, 2, 3, 0] => '╧',
        [3, 2, 3, 1] => '╪',
        [3, 2, 3, 2] => '╪',

        other => panic!("no box-drawing character for weight combination {:?}", other),
    }
}

type TileCoord = (usize, usize);
type GridDimensions = (usize, usize);
type Grid = Vec<Vec<Wires>>;
type PoweredTiles = Vec<Vec<bool>>;

#[derive(Clone, Copy)]
struct Wires {
    right: bool,
    up: bool,
    left: bool,
    down: bool,
}

impl Wires {
    fn new(right: bool, up: bool, left: bool, down: bool) -> Wires {
        Wires { right, up, left, down }
    }
}

fn build_grid() -> (Grid, PoweredTiles, TileCoord, GridDimensions, bool) {
    let width = 5;
    let height = 5;
    let wrapping = false;
    let source = (0, 0);
    let grid = vec![
        vec![
            Wires::new(true, false, false, true),
            Wires::new(true, false, true, true),
            Wires::new(true, false, true, true),
            Wires::new(true, false, true, false),
            Wires::new(false, false, true, true),
        ],
        vec![
            Wires::new(false, true, false, true),
            Wires::new(false, true, false, true),
            Wires::new(true, true, false, false),
            Wires::new(false, false, true, true),
            Wires::new(false, true, false, true),
        ],
        vec![
            Wires::new(false, true, false, true),
            Wires::new(false, true, false, false),
            Wires::new(false, false, true, true),
            Wires::new(false, true, true, false),
            Wires::new(false, true, false, true),
        ],
        vec![
            Wires::new(false, true, false, true),
            Wires::new(true, false, false, true),
            Wires::new(false, true, true, true),
            Wires::new(true, false, false, false),
            Wires::new(false, true, true, false),
        ],
        vec![
            Wires::new(false, true, false, false),
            Wires::new(false, true, false, false),
            Wires::new(true, true, false, false),
            Wires::new(true, false, true, false),
            Wires::new(false, false, true, false),
        ],
    ];
    let powered_tiles = vec![
        vec![true, true, true, true, true],
        vec![true, true, true, true, true],
        vec![true, true, false, true, true],
        vec![true, false, false, true, true],
        vec![true, false, false, false, false],
    ];
    (grid, powered_tiles, source, (width, height), wrapping)
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

/// Addressing a tile's contents can be done relative to the tile: the tile
/// is given as `(tile_x, tile_y)`, and the offset within it as a
/// quarter-step `offset_x` across (0..=4) and a half-step `offset_y` down
/// (0..=2), for a fixed 5 columns (one shared junction column + 3 center
/// columns) by 3 rows (one shared border row + 1 center row), overlapping
/// by one step with each neighbouring tile to merge borders between them.
fn tile_offset_to_coord((tile_x, tile_y): TileCoord, (offset_x, offset_y): Offset) -> Coord {
    let x = GRID_OFFSET_X + 4 * tile_x + offset_x;
    let y = GRID_OFFSET_Y + 2 * tile_y + offset_y;
    Coord::new(x, y)
}

// Helper functions name specific coordinates within a tile's footprint:
//
//             top_mid     a = center_left
//                │        b = center_mid
//                ▼        c = center_right
// top_left ───►┌───┐
// left_side ──►│abc│
//              └───┘
fn top_left(tile_coord: TileCoord) -> Coord {
    tile_offset_to_coord(tile_coord, (0, 0))
}
fn top_mid(tile_coord: TileCoord) -> Coord {
    tile_offset_to_coord(tile_coord, (2, 0))
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
/// `new()`, not any fixed constant. A real board's size is only known at
/// runtime. Also stores the frame's bounds.
struct Canvas {
    cells: Vec<Vec<Cell>>,
    frame_left: usize,
    frame_top: usize,
    frame_right: usize,
    frame_bottom: usize,
}

impl Canvas {
    fn new((tile_width, tile_height): GridDimensions) -> Canvas {
        let board_width = 4 * tile_width + 1;
        let board_height = 2 * tile_height + 1;
        let canvas_width = board_width + 2 * FRAME_MARGIN_X + 2 + 2 * CANVAS_MARGIN_X;
        let canvas_height = board_height + 2 * FRAME_MARGIN_Y + 2 + 2 * CANVAS_MARGIN_Y;
        Canvas {
            cells: vec![vec![Cell::Segment(BLANK); canvas_width]; canvas_height],
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
}

/// Draws a line of the given weight from `start` to `end`, inclusive, along
/// whichever of column or row they share. `start` and `end` don't need to
/// be given in order. Each endpoint only gets the arm pointing back into
/// the line, not the one pointing past it. Panics if `start` and `end` are
/// neither on the same row nor the same column.
fn draw_line(canvas: &mut Canvas, start: impl Into<Coord>, end: impl Into<Coord>, weight: u8) {
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
            let right = if x == end.x { NONE } else { weight };
            let left = if x == start.x { NONE } else { weight };
            let code = [right, NONE, left, NONE];
            canvas.draw_code((x, start.y), code);
        }
    } else if start.x == end.x {
        for y in start.y..=end.y {
            let up = if y == start.y { NONE } else { weight };
            let down = if y == end.y { NONE } else { weight };
            let code = [NONE, up, NONE, down];
            canvas.draw_code((start.x, y), code);
        }
    } else {
        panic!("draw_line only supports horizontal or vertical lines");
    }
}

/// Draws the fixed tile borders between cells, independent of any wire.
fn draw_grid_lines(canvas: &mut Canvas, (width, height): GridDimensions) {
    for tile_y in 0..=height {
        draw_line(canvas, top_left((0, tile_y)), top_left((width, tile_y)), LIGHT);
    }
    for tile_x in 0..=width {
        draw_line(canvas, top_left((tile_x, 0)), top_left((tile_x, height)), LIGHT);
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
fn draw_wires_and_endpoints(
    canvas: &mut Canvas,
    grid: &Grid,
    powered_tiles: &PoweredTiles,
    (width, height): GridDimensions,
) {
    for tile_y in 0..height {
        for tile_x in 0..width {
            let wires = grid[tile_y][tile_x];
            let is_powered = powered_tiles[tile_y][tile_x];
            let weight = if is_powered { HEAVY } else { LIGHT };
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

/// Draws barrier walls, fixed obstacles blocking a wire connection, in
/// double weight across the border they occupy. A wrapping grid has no
/// outer boundary, so this draws nothing; otherwise it forms the full
/// outer boundary ring.
fn draw_barriers(canvas: &mut Canvas, (width, height): GridDimensions, wrapping: bool) {
    if wrapping {
        return;
    }
    draw_line(canvas, top_left((0, 0)), top_left((width, 0)), DOUBLE);
    draw_line(canvas, top_left((0, height)), top_left((width, height)), DOUBLE);
    draw_line(canvas, top_left((0, 0)), top_left((0, height)), DOUBLE);
    draw_line(canvas, top_left((width, 0)), top_left((width, height)), DOUBLE);
}

/// Draws the outer presentation frame, offset from the grid lines by
/// FRAME_MARGIN_X/FRAME_MARGIN_Y and from the canvas edge by
/// CANVAS_MARGIN_X/CANVAS_MARGIN_Y.
fn draw_frame(canvas: &mut Canvas) {
    let left = canvas.frame_left;
    let top = canvas.frame_top;
    let right = canvas.frame_right;
    let bottom = canvas.frame_bottom;
    draw_line(canvas, (left, top), (right, top), LIGHT);
    draw_line(canvas, (left, bottom), (right, bottom), LIGHT);
    draw_line(canvas, (left, top), (left, bottom), LIGHT);
    draw_line(canvas, (right, top), (right, bottom), LIGHT);
}

/// Flattens the drawn canvas into one line of text per screen row.
fn flatten_to_lines(canvas: &Canvas) -> Vec<String> {
    let mut lines: Vec<String> = Vec::with_capacity(canvas.height());
    for y in 0..canvas.height() {
        let mut line = String::with_capacity(canvas.width());
        for x in 0..canvas.width() {
            line.push(canvas.char_at((x, y)));
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
        assert_eq!(glyph([LIGHT, LIGHT, LIGHT, LIGHT]), '┼');
    }

    #[test]
    fn double_corner() {
        // right=double, up=none, left=none, down=double -> top-left
        // corner of a double-line box.
        assert_eq!(glyph([DOUBLE, NONE, NONE, DOUBLE]), '╔');
    }

    #[test]
    fn combine_takes_elementwise_max() {
        let a = [NONE, LIGHT, NONE, LIGHT];
        let b = [HEAVY, NONE, HEAVY, NONE];
        assert_eq!(combine(a, b), [HEAVY, LIGHT, HEAVY, LIGHT]);
    }

    #[test]
    fn heavy_downgrades_to_light() {
        // There's no Unicode glyph that mixes DOUBLE and HEAVY,
        // so in that case HEAVY turns into LIGHT.
        let glyph_with_heavy = glyph([HEAVY, DOUBLE, HEAVY, DOUBLE]);
        let glyph_with_light = glyph([LIGHT, DOUBLE, LIGHT, DOUBLE]);
        assert_eq!(glyph_with_heavy, glyph_with_light);
    }

    #[test]
    fn draw_line_normalizes_reversed_endpoints() {
        let start = (0, 0);
        let end = (3, 0);

        let mut natural = Canvas::new((2, 2));
        draw_line(&mut natural, start, end, LIGHT);

        let mut reversed = Canvas::new((2, 2));
        draw_line(&mut reversed, end, start, LIGHT);

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
        draw_line(&mut canvas, (0, 0), (3, 3), LIGHT);
    }

    #[test]
    fn draw_line_start_equals_end_is_noop() {
        let mut canvas = Canvas::new((2, 2));
        draw_line(&mut canvas, (1, 1), (1, 1), LIGHT);
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
        assert_tile_boundary_shared((0, 0), (1, 0), (4, 0));
    }

    #[test]
    fn tile_boundary_shared_y() {
        assert_tile_boundary_shared((0, 0), (0, 1), (0, 2));
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
    fn render_board_matches_reference() {
        let expected = "\
┌───────────────────────┐\n\
│ ╔═══╤═══╤═══╤═══╤═══╗ │\n\
│ ║▐🬰▌┿━┳━┿━┳━┿━━━┿━┓ ║ │\n\
│ ╟─╂─┼─╂─┼─╂─┼───┼─╂─╢ │\n\
│ ║ ┃ │ ┃ │ ┗━┿━┓ │ ┃ ║ │\n\
│ ╟─╂─┼─╂─┼───┼─╂─┼─╂─╢ │\n\
│ ║ ┃ │▐█▌├─┐ ┝━┛ │ ┃ ║ │\n\
│ ╟─╂─┼───┼─┼─┼───┼─╂─╢ │\n\
│ ║ ┃ │ ┌─┼─┤ │▐█▌┿━┛ ║ │\n\
│ ╟─╂─┼─┼─┼─┼─┼───┼───╢ │\n\
│ ║▐█▌│⢸⣿⡇│ └─┼───┼⢸⣿⡇║ │\n\
│ ╚═══╧═══╧═══╧═══╧═══╝ │\n\
└───────────────────────┘";
        assert_eq!(render_board(), expected);
    }
}

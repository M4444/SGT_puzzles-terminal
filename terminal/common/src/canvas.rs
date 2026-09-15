//! Drawing shared by every game. Lines, text and marks go onto a
//! canvas, which is then flattened into lines of text for the terminal.

use crossterm::style::{Attribute, SetAttribute};
use std::cmp::max;

/// How strongly a line is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Weight {
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
pub struct Coord {
    pub x: usize,
    pub y: usize,
}

impl Coord {
    pub fn new(x: usize, y: usize) -> Coord {
        Coord { x, y }
    }

    pub fn shifted_by(self, offset: Coord) -> Coord {
        Coord::new(self.x + offset.x, self.y + offset.y)
    }
}

impl From<(usize, usize)> for Coord {
    fn from((x, y): (usize, usize)) -> Coord {
        Coord::new(x, y)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

impl Size {
    pub fn new(width: usize, height: usize) -> Size {
        Size { width, height }
    }
}

/// A rectangular area of cells.
#[derive(Clone, Copy, Debug)]
pub struct Rect {
    top_left: Coord,
    size: Size,
}

impl Rect {
    pub fn new(top_left: Coord, size: Size) -> Rect {
        Rect { top_left, size }
    }

    /// A one cell tall rectangle.
    pub fn row(start: Coord, length: usize) -> Rect {
        Rect::new(start, Size::new(length, 1))
    }

    /// The rectangle spanning both corners, each of which it covers.
    pub fn from_corners(top_left: Coord, bottom_right: Coord) -> Rect {
        Rect::new(
            top_left,
            Size::new(bottom_right.x - top_left.x + 1, bottom_right.y - top_left.y + 1),
        )
    }

    pub fn left(self) -> usize {
        self.top_left.x
    }

    pub fn top(self) -> usize {
        self.top_left.y
    }

    pub fn right(self) -> usize {
        assert!(self.size.width > 0, "Rect::right() called on an empty rectangle");
        self.top_left.x + self.size.width - 1
    }

    pub fn bottom(self) -> usize {
        assert!(self.size.height > 0, "Rect::bottom() called on an empty rectangle");
        self.top_left.y + self.size.height - 1
    }

    pub fn shifted_by(self, offset: Coord) -> Rect {
        Rect::new(self.top_left.shifted_by(offset), self.size)
    }

    /// The cells the rectangle encloses, excluding its own border.
    /// One too narrow or short to enclose anything gives an empty
    /// rectangle.
    pub fn interior(self) -> Rect {
        Rect::new(
            Coord::new(self.top_left.x + 1, self.top_left.y + 1),
            Size::new(self.size.width.saturating_sub(2), self.size.height.saturating_sub(2)),
        )
    }

    /// Whether the coordinate falls inside the rectangle. An empty one
    /// contains nothing.
    pub fn contains(self, coord: Coord) -> bool {
        coord.x >= self.left()
            && coord.x - self.left() < self.size.width
            && coord.y >= self.top()
            && coord.y - self.top() < self.size.height
    }

    /// Every cell the rectangle covers, row by row.
    pub fn coords(self) -> impl Iterator<Item = Coord> {
        let Coord { x: left, y: top } = self.top_left;
        let Size { width, height } = self.size;

        (top..top + height).flat_map(move |y| (left..left + width).map(move |x| Coord::new(x, y)))
    }
}

/// Gap between the board and the canvas edge.
pub const CANVAS_MARGIN_X: usize = 0;
pub const CANVAS_MARGIN_Y: usize = 0;

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
pub enum Mark {
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
pub struct Canvas {
    cells: Vec<Vec<Cell>>,
    /// How each position renders, independent of `cells`' own content.
    marks: Vec<Vec<Marks>>,
}

impl Canvas {
    pub fn new(board_rect: Rect, menu_size: Size) -> Canvas {
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

    pub fn draw_char(&mut self, coord: impl Into<Coord>, c: char) {
        let coord = coord.into();
        self.cells[coord.y][coord.x] = Cell::Marker(c);
    }

    /// Draws each character of `text` in order, starting at `start` and
    /// advancing one column per character. Characters past the canvas's
    /// right edge are dropped.
    pub fn draw_text(&mut self, start: impl Into<Coord>, text: &str) {
        let start = start.into();
        for (offset_x, c) in text.chars().enumerate() {
            let x = start.x + offset_x;
            if x >= self.width() {
                break;
            }
            self.draw_char((x, start.y), c);
        }
    }

    pub fn char_at(&self, coord: impl Into<Coord>) -> char {
        let coord = coord.into();
        match self.cells[coord.y][coord.x] {
            Cell::Segment(code) => glyph(code),
            Cell::Marker(c) => c,
        }
    }

    pub fn mark(&mut self, coord: impl Into<Coord>, mark: Mark) {
        let coord = coord.into();
        let marks = &mut self.marks[coord.y][coord.x];

        match mark {
            Mark::Bold => marks.bold = true,
            Mark::Dimmed => marks.dimmed = true,
            Mark::Reversed => marks.reversed = true,
        }
    }

    pub fn mark_region(&mut self, coords: impl IntoIterator<Item = Coord>, mark: Mark) {
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

    pub fn toggle_reversed_region(&mut self, coords: impl IntoIterator<Item = Coord>) {
        for coord in coords {
            self.toggle_reversed(coord);
        }
    }

    pub fn is_reversed(&self, coord: impl Into<Coord>) -> bool {
        let coord = coord.into();
        self.marks[coord.y][coord.x].reversed
    }

    /// Draws text, marking every cell it covers.
    pub fn draw_text_marked(&mut self, start: impl Into<Coord>, text: &str, mark: Option<Mark>) {
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
pub fn draw_line(
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

/// The outline runs through the centers of `Rect`'s outermost cells.
pub fn draw_rect_outline(canvas: &mut Canvas, rect: Rect, weight: Weight) {
    let (left, top, right, bottom) = (rect.left(), rect.top(), rect.right(), rect.bottom());

    draw_line(canvas, (left, top), (right, top), weight);
    draw_line(canvas, (left, bottom), (right, bottom), weight);
    draw_line(canvas, (left, top), (left, bottom), weight);
    draw_line(canvas, (right, top), (right, bottom), weight);
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
pub fn flatten_to_lines(canvas: &Canvas, terminal_columns: usize) -> Vec<String> {
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

    /// A canvas of the given size, with no menu beside it.
    fn canvas(width: usize, height: usize) -> Canvas {
        Canvas::new(Rect::new(Coord::new(0, 0), Size::new(width, height)), Size::new(0, 0))
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

        let mut natural = canvas(4, 4);
        draw_line(&mut natural, start, end, Weight::Light);

        let mut reversed = canvas(4, 4);
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
        let mut canvas = canvas(4, 4);
        draw_line(&mut canvas, (0, 0), (3, 3), Weight::Light);
    }

    #[test]
    fn draw_line_start_equals_end_is_noop() {
        let mut canvas = canvas(4, 4);
        draw_line(&mut canvas, (1, 1), (1, 1), Weight::Light);
        assert_eq!(canvas.char_at((1, 1)), ' ');
    }

    /// A board 25 columns wide and a menu 60 wide make a canvas 85 wide.
    /// The board is the taller of the two, so it sets the height.
    #[test]
    fn canvas_new_computes_expected_size() {
        let board_rect = Rect::new(Coord::new(0, 0), Size::new(25, 14));
        let canvas = Canvas::new(board_rect, Size::new(60, 8));
        assert_eq!(canvas.width(), 85);
        assert_eq!(canvas.height(), 14);
    }

    #[test]
    #[should_panic(expected = "draw_code() called on a marked cell")]
    fn draw_code_panics_on_marked_cell() {
        let mut canvas = canvas(1, 1);
        canvas.draw_char((0, 0), 'x');
        canvas.draw_code((0, 0), BLANK);
    }

    #[test]
    fn flatten_wraps_reversed_cells_in_escape_codes() {
        let mut canvas = canvas(1, 1);
        canvas.mark((0, 0), Mark::Reversed);

        let lines = flatten_to_lines(&canvas, usize::MAX);
        assert!(lines[0].starts_with(&SetAttribute(Attribute::Reverse).to_string()));
        assert!(lines[0].contains(&SetAttribute(Attribute::NoReverse).to_string()));
    }

    #[test]
    fn flatten_wraps_dimmed_cells_in_escape_codes() {
        let mut canvas = canvas(1, 1);
        canvas.mark_region([Coord::new(0, 0)], Mark::Dimmed);

        let lines = flatten_to_lines(&canvas, usize::MAX);
        assert!(lines[0].starts_with(&SetAttribute(Attribute::Dim).to_string()));
        assert!(lines[0].contains(&SetAttribute(Attribute::NormalIntensity).to_string()));
    }

    #[test]
    fn marking_reversed_twice_stays_reversed() {
        let mut canvas = canvas(1, 1);
        canvas.mark((0, 0), Mark::Reversed);
        canvas.mark((0, 0), Mark::Reversed);

        assert!(canvas.is_reversed((0, 0)));
    }

    #[test]
    fn toggle_reversed_twice_cancels_out() {
        let mut canvas = canvas(1, 1);
        canvas.toggle_reversed((0, 0));
        canvas.toggle_reversed((0, 0));

        assert!(!canvas.is_reversed((0, 0)));
    }

    #[test]
    fn toggle_reversed_cancels_a_prior_reversed_mark() {
        let mut canvas = canvas(1, 1);
        canvas.mark((0, 0), Mark::Reversed);
        canvas.toggle_reversed((0, 0));

        assert!(!canvas.is_reversed((0, 0)));
    }

    /// A line cut at the terminal's edge still ends with its resets, so
    /// reverse video can't run on past it.
    #[test]
    fn flatten_cuts_lines_at_the_terminal_edge_and_still_closes_them() {
        let mut canvas = canvas(2, 1);
        canvas.mark_region([Coord::new(0, 0), Coord::new(1, 0)], Mark::Reversed);

        let lines = flatten_to_lines(&canvas, 1);

        let reverse = SetAttribute(Attribute::Reverse);
        let no_reverse = SetAttribute(Attribute::NoReverse);
        assert_eq!(lines[0], format!("{reverse}{}{no_reverse}", canvas.char_at((0, 0))));
    }
}

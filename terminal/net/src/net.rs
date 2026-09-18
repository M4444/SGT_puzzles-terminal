//! Generating and reading Net puzzles, with the menu tabs Net offers.

use crate::ffi::window_offset;
use crate::menu::{self, Action, BodySpec, ButtonSpec, LegendEntry, LegendGroup, TabSpec};
use crate::render::{CursorStyle, LockStyle, Styles};
use common::board::{Grid, GridDimensions, TileCoord};
use common::ffi::{Preset, RawDrawing, RawDrawingApi, RawGame};
use common::session::{Frontend, Session};
use std::ffi::{CStr, c_char, c_int};

unsafe extern "C" {
    #[link_name = "thegame"]
    static THEGAME: RawGame;
    #[link_name = "terminal_drawing_api"]
    static TERMINAL_DRAWING_API: RawDrawingApi;
}

/// Which directions a tile has a wire pointing in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) struct Wires {
    pub right: bool,
    pub up: bool,
    pub left: bool,
    pub down: bool,
}

/// Bit flags within a direction nibble, matching net.c's `R`/`U`/`L`/`D`
/// `#define`s.
const RIGHT: u8 = 0x1;
const UP: u8 = 0x2;
const LEFT: u8 = 0x4;
const DOWN: u8 = 0x8;

/// Bit flag within a tile's byte marking it locked, matching net.c's
/// `LOCKED` `#define`.
const LOCKED_BIT: u8 = 0x10;

impl Wires {
    fn from_bits(bits: u8) -> Wires {
        Wires {
            right: bits & RIGHT != 0,
            up: bits & UP != 0,
            left: bits & LEFT != 0,
            down: bits & DOWN != 0,
        }
    }
}

/// Which of a tile's four edges carry a barrier, a wall no wire can
/// cross. Both tiles either side of a wall record it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) struct Barriers {
    pub right: bool,
    pub up: bool,
    pub left: bool,
    pub down: bool,
}

impl Barriers {
    fn from_bits(bits: u8) -> Barriers {
        Barriers {
            right: bits & RIGHT != 0,
            up: bits & UP != 0,
            left: bits & LEFT != 0,
            down: bits & DOWN != 0,
        }
    }
}

/// A single tile's wires and barriers, and whether it's currently
/// powered or locked.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) struct Tile {
    pub wires: Wires,
    pub barriers: Barriers,
    pub powered: bool,
    pub locked: bool,
}

pub(crate) type Tiles = Grid<Tile>;

/// The keyboard cursor's position and whether it's currently shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Cursor {
    pub position: TileCoord,
    pub visible: bool,
}

/// A Net puzzle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NetPuzzle {
    pub dimensions: GridDimensions,
    pub tiles: Tiles,
    pub cursor: Cursor,
    pub source: TileCoord,
    pub origin: TileCoord,
    pub status: String,
}

/// Mirrors `struct live_state` (net.c), what Net hands over through the
/// drawing API's `emit_state`.
#[repr(C)]
struct RawLiveState {
    active: *const u8,
    tiles: *const u8,
    barriers: *const u8,
    width: c_int,
    height: c_int,
    cur_x: c_int,
    cur_y: c_int,
    cur_visible: bool,
    source_x: c_int,
    source_y: c_int,
    org_x: c_int,
    org_y: c_int,
}

/// Called from net.c's `game_redraw`, through our own `drawing_api`
/// (`terminal_drawing_api`), with the tiles and the `active` map
/// `compute_active` just produced.
#[unsafe(no_mangle)]
extern "C" fn rust_emit_state(dr: *mut RawDrawing, data: *const RawLiveState) {
    let data = unsafe { &*data };
    let width = data.width as usize;
    let height = data.height as usize;
    let frontend = unsafe { &mut *((*dr).handle as *mut Frontend<NetPuzzle>) };

    let raw_active = unsafe { std::slice::from_raw_parts(data.active, width * height) };
    let raw_tiles = unsafe { std::slice::from_raw_parts(data.tiles, width * height) };
    let raw_barriers = unsafe { std::slice::from_raw_parts(data.barriers, width * height) };

    let tiles: Vec<Tile> = raw_tiles
        .iter()
        .zip(raw_active.iter())
        .zip(raw_barriers.iter())
        .map(|((&bits, &active), &barriers)| Tile {
            wires: Wires::from_bits(bits),
            barriers: Barriers::from_bits(barriers),
            powered: active != 0,
            locked: bits & LOCKED_BIT != 0,
        })
        .collect();

    frontend.puzzle = Some(NetPuzzle {
        dimensions: (width, height),
        tiles: tiles.chunks(width).map(|row| row.to_vec()).collect(),
        cursor: Cursor {
            position: (data.cur_x as usize, data.cur_y as usize),
            visible: data.cur_visible,
        },
        source: (data.source_x as usize, data.source_y as usize),
        origin: (data.org_x as usize, data.org_y as usize),
        status: String::new(),
    });
}

/// Called from net.c's `status_bar`, right after `rust_emit_state` within
/// the same `game_redraw`.
#[unsafe(no_mangle)]
extern "C" fn rust_status_bar(dr: *mut RawDrawing, text: *const c_char) {
    let frontend = unsafe { &mut *((*dr).handle as *mut Frontend<NetPuzzle>) };
    let text = unsafe { CStr::from_ptr(text) }.to_string_lossy().into_owned();

    frontend.puzzle.as_mut().expect("emit_state was not called").status = text;
}

/// Net's mouse inputs.
const MOUSE_CONTROLS: &[LegendEntry] = &[
    LegendEntry { input: "Left button", description: "rotate tile 90° counter clockwise" },
    LegendEntry { input: "Right button", description: "rotate tile 90° clockwise" },
    LegendEntry { input: "Middle button", description: "lock / unlock tile" },
];

/// Net's keyboard inputs.
const KEYBOARD_CONTROLS: &[LegendEntry] = &[
    LegendEntry { input: "Arrows", description: "move cursor" },
    LegendEntry { input: "Ctrl + arrows", description: "move source tile" },
    LegendEntry { input: "Shift + arrows", description: "move origin (wrapping)" },
    LegendEntry { input: "Ctrl + Shift + arrows", description: "move source and origin" },
    LegendEntry { input: "A / Enter", description: "rotate tile 90° counter clockwise" },
    LegendEntry { input: "D", description: "rotate tile 90° clockwise" },
    LegendEntry { input: "F", description: "rotate tile 180°" },
    LegendEntry { input: "S / Space", description: "lock / unlock tile" },
    LegendEntry { input: "J", description: "jumble unlocked tiles" },
];

const GAME_CONTROLS: &[LegendGroup] = &[
    LegendGroup { label: Some("Mouse:"), entries: MOUSE_CONTROLS },
    LegendGroup { label: Some("Keyboard:"), entries: KEYBOARD_CONTROLS },
];

/// The choices only Net's menu offers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NetAction {
    SetCursorStyle(CursorStyle),
    SetLockStyle(LockStyle),
}

/// Net's menu tabs, the usual ones with its two style tabs among them.
pub(crate) fn tab_specs(presets: &[Preset]) -> Vec<TabSpec<NetAction>> {
    let styles = vec![
        TabSpec { name: Some("Cursor Style"), body: BodySpec::Choices(cursor_styles()) },
        TabSpec { name: Some("Lock Style"), body: BodySpec::Choices(lock_styles()) },
    ];

    menu::tabs(presets, styles, GAME_CONTROLS)
}

fn cursor_styles() -> Vec<ButtonSpec<NetAction>> {
    let choice = |style| Action::Game(NetAction::SetCursorStyle(style));

    vec![
        ButtonSpec::new(choice(CursorStyle::Outline), "Outline"),
        ButtonSpec::new(choice(CursorStyle::ReverseTileCenter), "Center"),
        ButtonSpec::new(choice(CursorStyle::ReverseTileFull), "Full"),
    ]
}

fn lock_styles() -> Vec<ButtonSpec<NetAction>> {
    let choice = |style| Action::Game(NetAction::SetLockStyle(style));

    vec![
        ButtonSpec::new(choice(LockStyle::ReverseTileConnected), "Merged"),
        ButtonSpec::new(choice(LockStyle::ReverseTileCenter), "Center"),
        ButtonSpec::new(choice(LockStyle::ReverseTileFull), "Full"),
    ]
}

/// Whether the board is already drawn with the style a choice sets.
pub(crate) fn is_current(action: NetAction, styles: Styles) -> bool {
    match action {
        NetAction::SetCursorStyle(style) => style == styles.cursor,
        NetAction::SetLockStyle(style) => style == styles.lock,
    }
}

/// Starts a game of Net.
pub(crate) fn new_session() -> Session<NetPuzzle> {
    unsafe { Session::new(&THEGAME, &TERMINAL_DRAWING_API) }
}

/// Sends one mouse button press on the given tile, converting it into
/// the pixel coordinates net.c's own click handling expects. Returns
/// `false` if it signalled quit.
pub(crate) fn click_tile(
    session: &mut Session<NetPuzzle>,
    (tile_x, tile_y): TileCoord,
    button: c_int,
) -> bool {
    let tilesize = session.tilesize();
    let line_thick = line_thick(tilesize);
    let window_offset = unsafe { window_offset() };

    // Centers the click in the tile.
    let x = window_offset + line_thick + tile_x as c_int * tilesize + tilesize / 2;
    let y = window_offset + line_thick + tile_y as c_int * tilesize + tilesize / 2;

    session.process_click(x, y, button)
}

/// The given tiles, without the locked ones.
pub(crate) fn exclude_locked(
    session: &Session<NetPuzzle>,
    tile_coords: &[TileCoord],
) -> Vec<TileCoord> {
    tile_coords
        .iter()
        .copied()
        .filter(|&(x, y)| !session.puzzle().tiles[y][x].locked)
        .collect()
}

/// Computes net.c's `LINE_THICK`, always derived from the tile size
/// the same way regardless of platform.
fn line_thick(tilesize: c_int) -> c_int {
    (tilesize + 47) / 48
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Net offers five sizes, then the same five wrapping.
    #[test]
    fn presets_are_the_sizes_net_lists() {
        let titles: Vec<String> = new_session()
            .presets()
            .into_iter()
            .map(|preset| preset.title)
            .collect();

        assert_eq!(titles.len(), 10);
        assert_eq!(titles[0], "5x5");
        assert_eq!(titles[4], "13x11");
        assert_eq!(titles[5], "5x5 wrapping");
        assert_eq!(titles[9], "13x11 wrapping");
    }

    /// A fresh game starts on the first preset, and choosing another
    /// one starts a new game at its size.
    #[test]
    fn setting_a_preset_starts_a_game_at_its_size() {
        let mut session = new_session();
        let presets = session.presets();
        let nine_by_nine = presets
            .iter()
            .find(|preset| preset.title == "9x9")
            .expect("a 9x9 preset");

        assert_eq!(session.which_preset(), Some(presets[0].id));

        session.set_preset(nine_by_nine.id);

        assert_eq!(session.puzzle().dimensions, (9, 9));
        assert_eq!(session.which_preset(), Some(nine_by_nine.id));
    }
}

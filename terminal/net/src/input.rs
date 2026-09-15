//! Turning key presses and mouse clicks into menu actions and the
//! mid-end's key codes.

use crate::menu;
use crate::net::{self, NetPuzzle};
use crate::render;
use common::board::{TileCoord, tiles_at};
use common::session::Session;
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};

/// Raw key codes from puzzles.h's enum.
const CURSOR_UP: i32 = 0x0209;
const CURSOR_DOWN: i32 = 0x020A;
const CURSOR_LEFT: i32 = 0x020B;
const CURSOR_RIGHT: i32 = 0x020C;
const CURSOR_SELECT: i32 = 0x020D;
const MOD_CTRL: i32 = 0x1000;
const MOD_SHFT: i32 = 0x2000;

/// Key codes for the menu's actions. Restart has none of its own.
const UI_QUIT: i32 = 0x0210;
const UI_NEWGAME: i32 = 0x0211;
const UI_SOLVE: i32 = 0x0212;
const UI_UNDO: i32 = 0x0213;
const UI_REDO: i32 = 0x0214;

/// Raw mouse button codes from puzzles.h's enum.
const LEFT_BUTTON: i32 = 0x0200;
const MIDDLE_BUTTON: i32 = 0x0201;
const RIGHT_BUTTON: i32 = 0x0202;

/// Takes one key press or mouse event and ignores any other kind.
/// Returns `false` if it signalled quit.
pub(crate) fn take_event(
    event: Event,
    menu: &mut menu::Menu,
    session: &mut Session<NetPuzzle>,
    styles: &mut render::Styles,
) -> bool {
    match event {
        Event::Key(key) => take_key(key, menu, session, styles),
        Event::Mouse(mouse) => take_click(mouse, menu, session, styles),
        _ => true,
    }
}

/// Takes one key press. 'Tab' moves the focus, and everything else goes
/// to whichever of the menu and the board holds it. Returns `false` if
/// it signalled quit.
fn take_key(
    key: KeyEvent,
    menu: &mut menu::Menu,
    session: &mut Session<NetPuzzle>,
    styles: &mut render::Styles,
) -> bool {
    match key.code {
        KeyCode::Tab => {
            menu.focus_next();
            true
        }
        KeyCode::BackTab => {
            menu.focus_previous();
            true
        }
        _ => {
            if menu.has_focus() {
                match key.code {
                    KeyCode::Left => {
                        menu.cursor_left();
                        true
                    }
                    KeyCode::Right => {
                        menu.cursor_right();
                        true
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => match menu.press() {
                        Some(action) => take_action(action, session, styles),
                        None => true,
                    },
                    _ => true,
                }
            } else {
                // The board has focus.
                let mut button = match key.code {
                    KeyCode::Char(c) => c as i32,
                    KeyCode::Up => CURSOR_UP,
                    KeyCode::Down => CURSOR_DOWN,
                    KeyCode::Left => CURSOR_LEFT,
                    KeyCode::Right => CURSOR_RIGHT,
                    KeyCode::Enter => CURSOR_SELECT,
                    _ => return true,
                };

                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    button |= MOD_CTRL;
                }
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    button |= MOD_SHFT;
                }

                session.process_key(button)
            }
        }
    }
}

/// Takes one mouse button press. A left click goes to the menu first,
/// and anything the menu doesn't claim falls through to the board.
/// Returns `false` if it signalled quit.
fn take_click(
    mouse: MouseEvent,
    menu: &mut menu::Menu,
    session: &mut Session<NetPuzzle>,
    styles: &mut render::Styles,
) -> bool {
    let button = match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => LEFT_BUTTON,
        MouseEventKind::Down(MouseButton::Middle) => MIDDLE_BUTTON,
        MouseEventKind::Down(MouseButton::Right) => RIGHT_BUTTON,
        _ => return true,
    };
    // Clicking anything moves the focus back to the board.
    menu.clear_focus();

    let position = (mouse.column as usize, mouse.row as usize);
    if button == LEFT_BUTTON
        && let Some(action) = menu.click(position)
    {
        return take_action(action, session, styles);
    }

    let possible_tiles = tiles_at(position, session.puzzle().dimensions);
    match resolve_tile(&possible_tiles, button, session) {
        Some(tile) => net::click_tile(session, tile, button),
        None => true,
    }
}

/// Carries out a menu action. Returns `false` if it signalled quit.
fn take_action(
    action: menu::Action,
    session: &mut Session<NetPuzzle>,
    styles: &mut render::Styles,
) -> bool {
    match action {
        menu::Action::NewGame => session.process_key(UI_NEWGAME),
        menu::Action::Undo => session.process_key(UI_UNDO),
        menu::Action::Redo => session.process_key(UI_REDO),
        menu::Action::Solve => session.process_key(UI_SOLVE),
        menu::Action::Quit => session.process_key(UI_QUIT),
        menu::Action::Restart => {
            session.restart();
            true
        }
        menu::Action::SetCursorStyle(style) => {
            styles.cursor = style;
            true
        }
        menu::Action::SetLockStyle(style) => {
            styles.lock = style;
            true
        }
        menu::Action::SetPreset(id) => {
            session.set_preset(id);
            true
        }
    }
}

/// A click resolves to one tile either because it landed cleanly on
/// one, or (for anything but the lock-toggle button) because exactly
/// one of several overlapping candidates isn't locked, since a locked
/// tile can never be rotated.
fn resolve_tile(
    possible_tiles: &[TileCoord],
    button: i32,
    session: &Session<NetPuzzle>,
) -> Option<TileCoord> {
    if let [only] = possible_tiles {
        Some(*only)
    } else if button != MIDDLE_BUTTON {
        if let [only] = net::exclude_locked(session, possible_tiles).as_slice() {
            Some(*only)
        } else {
            None
        }
    } else {
        None
    }
}

//! Turning key presses and mouse clicks into menu actions and the
//! mid-end's key codes.

use crate::canvas::Coord;
use crate::menu;
use crate::session::Session;
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use std::ffi::c_int;

/// Raw key codes from puzzles.h's enum.
const CURSOR_UP: c_int = 0x0209;
const CURSOR_DOWN: c_int = 0x020A;
const CURSOR_LEFT: c_int = 0x020B;
const CURSOR_RIGHT: c_int = 0x020C;
const CURSOR_SELECT: c_int = 0x020D;
const MOD_CTRL: c_int = 0x1000;
const MOD_SHFT: c_int = 0x2000;

/// Key codes for the actions a front end offers. Restart has none.
const UI_QUIT: c_int = 0x0210;
const UI_NEWGAME: c_int = 0x0211;
const UI_SOLVE: c_int = 0x0212;
const UI_UNDO: c_int = 0x0213;
const UI_REDO: c_int = 0x0214;

/// Raw mouse button codes from puzzles.h's enum.
const LEFT_BUTTON: c_int = 0x0200;
pub const MIDDLE_BUTTON: c_int = 0x0201;
const RIGHT_BUTTON: c_int = 0x0202;

/// The code the mid-end knows a key press by, with its Ctrl and Shift
/// modifiers folded in. Keys the mid-end has no code for give `None`.
pub fn key_code(key: KeyEvent) -> Option<c_int> {
    let mut code = match key.code {
        KeyCode::Char(character) => character as c_int,
        KeyCode::Up => CURSOR_UP,
        KeyCode::Down => CURSOR_DOWN,
        KeyCode::Left => CURSOR_LEFT,
        KeyCode::Right => CURSOR_RIGHT,
        KeyCode::Enter => CURSOR_SELECT,
        _ => return None,
    };

    if key.modifiers.contains(KeyModifiers::CONTROL) {
        code |= MOD_CTRL;
    }
    if key.modifiers.contains(KeyModifiers::SHIFT) {
        code |= MOD_SHFT;
    }

    Some(code)
}

/// Takes one key press or mouse event and ignores any other kind. The
/// game's own menu actions go to `take_game_action`. A click the menu
/// doesn't claim goes to `click_board`. Returns `false` if it signalled
/// quit.
pub fn take_event<P, G: Copy>(
    event: Event,
    menu: &mut menu::Menu<G>,
    session: &mut Session<P>,
    take_game_action: impl FnOnce(G),
    click_board: impl FnOnce(&mut Session<P>, Coord, c_int) -> bool,
) -> bool {
    match event {
        Event::Key(key) => take_key(key, menu, session, take_game_action),
        Event::Mouse(mouse) => take_click(mouse, menu, session, take_game_action, click_board),
        _ => true,
    }
}

/// Takes one key press. 'Tab' moves the focus, and everything else goes
/// to whichever of the menu and the board holds it. Returns `false` if
/// it signalled quit.
fn take_key<P, G: Copy>(
    key: KeyEvent,
    menu: &mut menu::Menu<G>,
    session: &mut Session<P>,
    take_game_action: impl FnOnce(G),
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
                        Some(action) => take_action(action, session, take_game_action),
                        None => true,
                    },
                    _ => true,
                }
            } else {
                // The board has focus.
                match key_code(key) {
                    Some(button) => session.process_key(button),
                    None => true,
                }
            }
        }
    }
}

/// Takes one mouse button press. A left click goes to the menu first,
/// and anything the menu doesn't claim falls through to the board.
/// Returns `false` if it signalled quit.
fn take_click<P, G: Copy>(
    mouse: MouseEvent,
    menu: &mut menu::Menu<G>,
    session: &mut Session<P>,
    take_game_action: impl FnOnce(G),
    click_board: impl FnOnce(&mut Session<P>, Coord, c_int) -> bool,
) -> bool {
    let button = match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => LEFT_BUTTON,
        MouseEventKind::Down(MouseButton::Middle) => MIDDLE_BUTTON,
        MouseEventKind::Down(MouseButton::Right) => RIGHT_BUTTON,
        _ => return true,
    };
    // Clicking anything moves the focus back to the board.
    menu.clear_focus();

    let position = Coord::new(mouse.column as usize, mouse.row as usize);
    if button == LEFT_BUTTON
        && let Some(action) = menu.click(position)
    {
        return take_action(action, session, take_game_action);
    }

    click_board(session, position, button)
}

/// Carries out a menu action. Returns `false` if it signalled quit.
fn take_action<P, G>(
    action: menu::Action<G>,
    session: &mut Session<P>,
    take_game_action: impl FnOnce(G),
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
        menu::Action::Game(action) => {
            take_game_action(action);
            true
        }
        menu::Action::SetPreset(id) => {
            session.set_preset(id);
            true
        }
    }
}

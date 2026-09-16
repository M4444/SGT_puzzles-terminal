//! The mid-end's key and button codes, with the key presses that
//! produce them.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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
pub const UI_QUIT: c_int = 0x0210;
pub const UI_NEWGAME: c_int = 0x0211;
pub const UI_SOLVE: c_int = 0x0212;
pub const UI_UNDO: c_int = 0x0213;
pub const UI_REDO: c_int = 0x0214;

/// Raw mouse button codes from puzzles.h's enum.
pub const LEFT_BUTTON: c_int = 0x0200;
pub const MIDDLE_BUTTON: c_int = 0x0201;
pub const RIGHT_BUTTON: c_int = 0x0202;

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

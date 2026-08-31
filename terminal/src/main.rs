mod ffi;
mod menu;
mod net;
mod render;

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseButton,
    MouseEventKind,
};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use net::{Session, TileCoord};
use std::io::{stdout, Write};

/// Raw key codes from puzzles.h's enum.
const CURSOR_UP: i32 = 0x0209;
const CURSOR_DOWN: i32 = 0x020A;
const CURSOR_LEFT: i32 = 0x020B;
const CURSOR_RIGHT: i32 = 0x020C;
const CURSOR_SELECT: i32 = 0x020D;
const MOD_CTRL: i32 = 0x1000;
const MOD_SHFT: i32 = 0x2000;

/// Raw mouse button codes from puzzles.h's enum.
const LEFT_BUTTON: i32 = 0x0200;
const MIDDLE_BUTTON: i32 = 0x0201;
const RIGHT_BUTTON: i32 = 0x0202;

/// Restores the terminal on the way out, including on panic.
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        execute!(stdout(), DisableMouseCapture, Show).ok();
        disable_raw_mode().ok();
    }
}

fn main() {
    enable_raw_mode().expect("failed to enable raw mode");
    execute!(stdout(), EnableMouseCapture, Hide).ok();
    let _guard = TerminalGuard;

    let mut session = net::Session::new();
    let cursor_style = render::CursorStyle::default();
    let lock_style = render::LockStyle::default();

    loop {
        let dimensions = session.puzzle().dimensions;
        let menu = menu::Menu::new(render::grid_frame(dimensions).top_right());
        let output = render::render_game(session.puzzle(), cursor_style, lock_style, &menu);
        execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0)).ok();
        print!("{}\r\n", output.replace('\n', "\r\n"));
        stdout().flush().ok();

        let event = event::read().expect("failed to read input event");
        if let Event::Key(key_event) = event {
            let button = match key_event.code {
                KeyCode::Char(c) => Some(c as i32),
                KeyCode::Up => Some(CURSOR_UP),
                KeyCode::Down => Some(CURSOR_DOWN),
                KeyCode::Left => Some(CURSOR_LEFT),
                KeyCode::Right => Some(CURSOR_RIGHT),
                KeyCode::Enter => Some(CURSOR_SELECT),
                _ => None,
            };
            let mut button = button;
            if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                button = button.map(|button| button | MOD_CTRL);
            }
            if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                button = button.map(|button| button | MOD_SHFT);
            }
            if let Some(button) = button {
                if !session.process_key(button) {
                    break;
                }
            }
        } else if let Event::Mouse(mouse_event) = event {
            let button = match mouse_event.kind {
                MouseEventKind::Down(MouseButton::Left) => Some(LEFT_BUTTON),
                MouseEventKind::Down(MouseButton::Middle) => Some(MIDDLE_BUTTON),
                MouseEventKind::Down(MouseButton::Right) => Some(RIGHT_BUTTON),
                _ => None,
            };
            if let Some(button) = button {
                let position = (mouse_event.column as usize, mouse_event.row as usize);
                let possible_tiles = render::tiles_at(position, dimensions);
                if let Some(tile) = resolve_tile(&possible_tiles, button, &session) {
                    if !session.process_click(tile, button) {
                        break;
                    }
                }
            }
        }
    }
}

/// A click resolves to one tile either because it landed cleanly on
/// one, or (for anything but the lock-toggle button) because exactly
/// one of several overlapping candidates isn't locked, since a locked
/// tile can never be rotated.
fn resolve_tile(possible_tiles: &[TileCoord], button: i32, session: &Session) -> Option<TileCoord> {
    if let [only] = possible_tiles {
        Some(*only)
    } else if button != MIDDLE_BUTTON {
        if let [only] = session.exclude_locked(possible_tiles).as_slice() {
            Some(*only)
        } else {
            None
        }
    } else {
        None
    }
}

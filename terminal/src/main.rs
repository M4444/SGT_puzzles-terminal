mod ffi;
mod net;
mod render;

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseButton,
    MouseEventKind,
};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
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
        let board = render::render_board(session.puzzle(), cursor_style, lock_style);
        execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0)).ok();
        print!("{}\r\n", board.replace('\n', "\r\n"));
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
                if let Some(tile) = render::tile_at(position, session.puzzle().dimensions) {
                    if !session.process_click(tile, button) {
                        break;
                    }
                }
            }
        }
    }
}

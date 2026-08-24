mod ffi;
mod net;
mod render;

use crossterm::cursor::MoveTo;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
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

/// Restores the terminal on the way out, including on panic.
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        disable_raw_mode().ok();
    }
}

fn main() {
    enable_raw_mode().expect("failed to enable raw mode");
    let _guard = TerminalGuard;

    let mut session = net::Session::new();
    let cursor_style = render::CursorStyle::default();

    loop {
        let board = render::render_board(session.puzzle(), cursor_style);
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
            let button = if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                button.map(|button| button | MOD_CTRL)
            } else {
                button
            };
            if let Some(button) = button {
                if !session.process_key(button) {
                    break;
                }
            }
        }
    }
}

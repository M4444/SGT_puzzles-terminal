mod ffi;
mod input;
mod menu;
mod net;
mod render;

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event};
use crossterm::execute;
use crossterm::terminal::{self, Clear, ClearType, disable_raw_mode, enable_raw_mode};
use std::io::{Write, stdout};

/// Restores the terminal on the way out, including on panic.
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        execute!(stdout(), DisableMouseCapture, Show).ok();
        disable_raw_mode().ok();
        // Puts the shell's prompt on a fresh line.
        print!("\r\n");
        stdout().flush().ok();
    }
}

fn main() {
    enable_raw_mode().expect("failed to enable raw mode");
    execute!(stdout(), EnableMouseCapture, Hide).ok();
    let _guard = TerminalGuard;
    let (mut columns, mut rows) = terminal::size().expect("failed to query terminal size");

    let mut session = net::new_session();
    let mut menu = menu::Menu::new(&session.presets());

    let mut styles = render::Styles::default();

    let mut game_running = true;
    while game_running {
        let menu_state = menu::MenuState {
            styles,
            preset: session.which_preset(),
            can_undo: session.can_undo(),
            can_redo: session.can_redo(),
        };
        let output = render::render_game(
            session.puzzle(),
            session.wants_status_bar(),
            menu_state,
            &mut menu,
            columns.into(),
        );

        // Anything past the last row would scroll the top away.
        let output: Vec<&str> = output.lines().take(rows.into()).collect();

        execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0)).ok();
        print!("{}", output.join("\r\n"));
        stdout().flush().ok();

        let event = event::read().expect("failed to read input event");
        game_running = match event {
            Event::Resize(new_columns, new_rows) => {
                columns = new_columns;
                rows = new_rows;
                true
            }
            event => input::take_event(event, &mut menu, &mut session, &mut styles),
        };
    }
}

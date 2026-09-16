//! Setting the terminal up for a game and running its draw loop.

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

/// Draws the game and waits for an event until `take_event` reports
/// that the game is over. The terminal's width is passed to `render`.
/// Unlike the other events, resizes are handled here.
pub fn run<S>(
    state: &mut S,
    mut render: impl FnMut(&mut S, usize) -> String,
    mut take_event: impl FnMut(&mut S, Event) -> bool,
) {
    enable_raw_mode().expect("failed to enable raw mode");
    execute!(stdout(), EnableMouseCapture, Hide).ok();
    let _guard = TerminalGuard;
    let (mut columns, mut rows) = terminal::size().expect("failed to query terminal size");

    let mut game_running = true;
    while game_running {
        print_output(&render(state, columns.into()), rows.into());

        let event = event::read().expect("failed to read input event");
        game_running = match event {
            Event::Resize(new_columns, new_rows) => {
                columns = new_columns;
                rows = new_rows;
                true
            }
            event => take_event(state, event),
        };
    }
}

/// Clears the screen and prints the drawn game.
fn print_output(output: &str, rows: usize) {
    // Anything past the last row would scroll the top away.
    let lines: Vec<&str> = output.lines().take(rows).collect();

    execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0)).ok();
    print!("{}", lines.join("\r\n"));
    stdout().flush().ok();
}

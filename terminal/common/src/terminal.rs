//! Setting the terminal up for a game and running its draw loop, with
//! the hooks each game supplies.

use crate::board::{Board, Puzzle, draw_frame, draw_status_bar};
use crate::canvas::{Canvas, Coord, flatten_to_lines};
use crate::ffi::Preset;
use crate::input;
use crate::menu::{Menu, MenuState, TabSpec};
use crate::session::Session;
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event};
use crossterm::execute;
use crossterm::terminal::{self, Clear, ClearType, disable_raw_mode, enable_raw_mode};
use std::ffi::c_int;
use std::io::{Write, stdout};

/// A game's own tabs, board drawing, menu actions and board clicks.
/// The type that implements it holds any settings only that game has.
pub trait TerminalGame {
    /// The puzzle type the game's session carries.
    type Puzzle: Puzzle;
    /// The actions only the game's own menu tabs offer.
    type Action: Copy;

    /// The game's menu tabs.
    fn tab_specs(presets: &[Preset], can_solve: bool) -> Vec<TabSpec<Self::Action>>;

    /// Draws the board itself, without the frame or the menu around it.
    fn draw_board(&self, canvas: &mut Canvas, puzzle: &Self::Puzzle);

    /// The status bar's text, for a game that has one.
    fn status(_puzzle: &Self::Puzzle) -> &str {
        ""
    }

    /// Whether the game already uses the setting this action would
    /// apply.
    fn is_action_current(&self, action: Self::Action) -> bool;

    /// Carries out one of the game's own menu actions.
    fn take_action(&mut self, action: Self::Action);

    /// Takes a click the menu didn't claim. Returns `false` if it
    /// signalled quit.
    fn click_board(session: &mut Session<Self::Puzzle>, position: Coord, button: c_int) -> bool;
}

/// The front end's own state. It owns the game being played, its
/// session and its menu.
struct Frontend<T: TerminalGame> {
    game: T,
    session: Session<T::Puzzle>,
    menu: Menu<T::Action>,
}

impl<T: TerminalGame> Frontend<T> {
    fn render(&mut self, terminal_columns: usize) -> String {
        render_game(
            &self.game,
            self.session.puzzle(),
            self.session.wants_status_bar(),
            MenuState::new(&self.session),
            &mut self.menu,
            terminal_columns,
        )
    }

    fn take_event(&mut self, event: Event) -> bool {
        input::take_event(
            event,
            &mut self.menu,
            &mut self.session,
            |action| self.game.take_action(action),
            T::click_board,
        )
    }
}

/// Draws a game's board, the frame and status bar around it and the
/// menu beside it, then flattens the result to lines of text.
pub fn render_game<T: TerminalGame>(
    game: &T,
    puzzle: &T::Puzzle,
    wants_status_bar: bool,
    menu_state: MenuState,
    menu: &mut Menu<T::Action>,
    terminal_columns: usize,
) -> String {
    let board = Board::new(puzzle.dimensions(), wants_status_bar);
    let board_rect = board.rect();
    let mut canvas = Canvas::new(board_rect, menu.size());

    game.draw_board(&mut canvas, puzzle);
    // The frame is what marks the board as focused.
    if !menu.has_focus() {
        draw_frame(&mut canvas, board.frame);
    }
    if let Some(start) = board.status_bar_start {
        draw_status_bar(&mut canvas, start, T::status(puzzle));
    }

    menu.set_placement(board_rect, terminal_columns);
    menu.draw(&mut canvas, menu_state, |action| game.is_action_current(action));

    flatten_to_lines(&canvas, terminal_columns).join("\n")
}

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

/// Runs a game, drawing it and taking events until one of them quits
/// the game. Every event but a resize goes on to the front end.
pub fn run<T: TerminalGame>(game: T) {
    let session = Session::start();
    let menu = Menu::new(T::tab_specs(&session.presets(), session.can_solve()));
    let mut frontend = Frontend { game, session, menu };

    enable_raw_mode().expect("failed to enable raw mode");
    execute!(stdout(), EnableMouseCapture, Hide).ok();
    let _guard = TerminalGuard;
    let (mut columns, mut rows) = terminal::size().expect("failed to query terminal size");

    let mut game_running = true;
    while game_running {
        print_output(&frontend.render(columns.into()), rows.into());

        let event = event::read().expect("failed to read input event");
        game_running = match event {
            Event::Resize(new_columns, new_rows) => {
                columns = new_columns;
                rows = new_rows;
                true
            }
            event => frontend.take_event(event),
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

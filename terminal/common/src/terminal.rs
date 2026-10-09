//! Setting the terminal up for a game and running its draw loop, with
//! the hooks each game supplies.

use crate::board::{Board, Puzzle, draw_frame, draw_status_bar};
use crate::canvas::{Canvas, Coord, Size, flatten_to_lines};
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
use std::panic;
use std::thread;
use std::time::{Duration, Instant};

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

    /// Takes a click on the grid. Returns `false` if it signalled quit.
    fn click_board(session: &mut Session<Self::Puzzle>, position: Coord, button: c_int) -> bool;
}

/// The front end's own state. It owns the game being played, its
/// session, its menu and the board they are drawn on.
struct Frontend<T: TerminalGame> {
    game: T,
    session: Session<T::Puzzle>,
    /// The board as it was last drawn, or `None` before the first draw.
    board: Option<Board>,
    menu: Menu<T::Action>,
}

impl<T: TerminalGame> Frontend<T> {
    fn render(&mut self, terminal_columns: usize) -> String {
        let status = self
            .session
            .wants_status_bar()
            .then(|| T::status(self.session.puzzle()));
        let board = self
            .board
            .insert(Board::new(self.session.puzzle().dimensions(), status));

        render_game(
            &self.game,
            self.session.puzzle(),
            board,
            MenuState::new(&self.session),
            &mut self.menu,
            terminal_columns,
        )
    }

    fn take_event(&mut self, event: Event) -> bool {
        input::take_event(
            event,
            self.board.as_ref(),
            &mut self.menu,
            &mut self.session,
            |action| self.game.take_action(action),
            T::click_board,
        )
    }
}

/// Draws a board on its own, framed, with no session or menu behind
/// it, then prints it. A game uses this while its state is still
/// hardcoded.
pub fn preview<T: TerminalGame>(game: T, puzzle: &T::Puzzle) {
    let board = Board::new(puzzle.dimensions(), None);
    let mut canvas = Canvas::new(board.rect(), Size::new(0, 0));

    game.draw_board(&mut canvas, puzzle);
    draw_frame(&mut canvas, board.frame);

    for line in flatten_to_lines(&canvas, usize::MAX) {
        println!("{line}");
    }
}

/// Draws a game's board, the frame and status bar around it and the
/// menu beside it, then flattens the result to lines of text.
fn render_game<T: TerminalGame>(
    game: &T,
    puzzle: &T::Puzzle,
    board: &Board,
    menu_state: MenuState,
    menu: &mut Menu<T::Action>,
    terminal_columns: usize,
) -> String {
    let board_rect = board.rect();
    let mut canvas = Canvas::new(board_rect, menu.size());

    game.draw_board(&mut canvas, puzzle);
    // The frame is what marks the board as focused.
    if !menu.has_focus() {
        draw_frame(&mut canvas, board.frame);
    }
    draw_status_bar(&mut canvas, board);

    menu.set_placement(board_rect, terminal_columns);
    menu.draw(&mut canvas, menu_state, |action| game.is_action_current(action));

    flatten_to_lines(&canvas, terminal_columns).join("\n")
}

/// Gives the terminal back the settings the game took over.
fn restore_terminal() {
    execute!(stdout(), DisableMouseCapture, Show).ok();
    disable_raw_mode().ok();
    stdout().flush().ok();
}

/// Sets a hook that restores the terminal before a panic's message is
/// printed, which happens before unwinding reaches the guard below.
fn set_panic_hook() {
    let previous_hook = panic::take_hook();

    panic::set_hook(Box::new(move |panic_info| {
        restore_terminal();
        previous_hook(panic_info);
    }));
}

/// Restores the terminal on the way out, including on panic.
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore_terminal();

        // Puts the shell's prompt on a fresh line. A panic's message
        // ends with one of its own, so it needs no help.
        if !thread::panicking() {
            print!("\r\n");
            stdout().flush().ok();
        }
    }
}

/// How long to wait for input before passing time to the mid-end,
/// while it wants any.
const TICK_INTERVAL: Duration = Duration::from_millis(20);

/// Runs a game, drawing it and taking events until one of them quits
/// the game. Every event but a resize goes on to the front end. A
/// frame identical to the one on screen isn't printed again.
pub fn run<T: TerminalGame>(game: T) {
    let session = Session::start();
    let menu = Menu::new(T::tab_specs(&session.presets(), session.can_solve()));
    let mut frontend = Frontend { game, session, board: None, menu };

    set_panic_hook();
    enable_raw_mode().expect("failed to enable raw mode");
    execute!(stdout(), EnableMouseCapture, Hide).ok();
    let _guard = TerminalGuard;
    let (mut columns, mut rows) = terminal::size().expect("failed to query terminal size");

    // The frame last printed, or `None` before the first one
    // and after a resize, which can change what's on screen
    // without printing anything.
    let mut printed: Option<String> = None;
    // When time was last passed to the mid-end, or when it started
    // wanting ticks, or `None` while it doesn't.
    let mut last_tick: Option<Instant> = None;

    let mut game_running = true;
    while game_running {
        let output = frontend.render(columns.into());
        if printed.as_ref() != Some(&output) {
            print_output(&output, rows.into());
            printed = Some(output);
        }

        if frontend.session.is_timer_active() {
            let counted_from = *last_tick.get_or_insert_with(Instant::now);

            if !event::poll(TICK_INTERVAL).expect("failed to poll for input events") {
                let now = Instant::now();
                frontend.session.timer(now - counted_from);
                last_tick = Some(now);

                continue;
            }
        } else {
            last_tick = None;
        }

        let event = event::read().expect("failed to read input event");
        game_running = match event {
            Event::Resize(new_columns, new_rows) => {
                columns = new_columns;
                rows = new_rows;
                printed = None;
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

//! Plays Fifteen in the terminal.

mod fifteen;
mod input;
mod render;

use common::canvas::{Canvas, Coord};
use common::ffi::Preset;
use common::menu::TabSpec;
use common::session::Session;
use common::terminal::TerminalGame;
use fifteen::{FifteenAction, FifteenPuzzle};
use std::ffi::c_int;

/// Fifteen, which has no settings of its own.
struct Fifteen;

impl TerminalGame for Fifteen {
    type Puzzle = FifteenPuzzle;
    type Action = FifteenAction;

    fn tab_specs(presets: &[Preset], can_solve: bool) -> Vec<TabSpec<FifteenAction>> {
        fifteen::tab_specs(presets, can_solve)
    }

    fn draw_board(&self, canvas: &mut Canvas, puzzle: &FifteenPuzzle) {
        render::draw_board(canvas, puzzle);
    }

    fn status(puzzle: &FifteenPuzzle) -> &str {
        &puzzle.status
    }

    fn is_action_current(&self, action: FifteenAction) -> bool {
        match action {}
    }

    fn take_action(&mut self, action: FifteenAction) {
        match action {}
    }

    fn click_board(session: &mut Session<FifteenPuzzle>, position: Coord, button: c_int) -> bool {
        input::click_board(session, position, button)
    }
}

fn main() {
    common::terminal::run(Fifteen);
}

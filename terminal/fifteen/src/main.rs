//! Draws a hardcoded Fifteen board for the terminal.

mod fifteen;
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

    fn tab_specs(_presets: &[Preset], _can_solve: bool) -> Vec<TabSpec<FifteenAction>> {
        todo!()
    }

    fn draw_board(&self, canvas: &mut Canvas, puzzle: &FifteenPuzzle) {
        render::draw_board(canvas, puzzle);
    }

    fn is_action_current(&self, action: FifteenAction) -> bool {
        match action {}
    }

    fn take_action(&mut self, action: FifteenAction) {
        match action {}
    }

    fn click_board(
        _session: &mut Session<FifteenPuzzle>,
        _position: Coord,
        _button: c_int,
    ) -> bool {
        todo!()
    }
}

fn main() {
    common::terminal::preview(Fifteen, &fifteen::hardcoded_game());
}

mod pegs;
mod render;

use common::canvas::{Canvas, Coord};
use common::ffi::Preset;
use common::menu::TabSpec;
use common::session::Session;
use common::terminal::TerminalGame;
use pegs::{PegsAction, PegsPuzzle};
use std::ffi::c_int;

/// Pegs, which has no settings of its own.
struct Pegs;

impl TerminalGame for Pegs {
    type Puzzle = PegsPuzzle;
    type Action = PegsAction;

    fn tab_specs(presets: &[Preset], can_solve: bool) -> Vec<TabSpec<PegsAction>> {
        pegs::tab_specs(presets, can_solve)
    }

    fn draw_board(&self, canvas: &mut Canvas, puzzle: &PegsPuzzle) {
        render::draw_board(canvas, puzzle);
    }

    fn is_action_current(&self, action: PegsAction) -> bool {
        match action {}
    }

    fn take_action(&mut self, action: PegsAction) {
        match action {}
    }

    /// Clicks on the board do nothing.
    fn click_board(_session: &mut Session<PegsPuzzle>, _position: Coord, _button: c_int) -> bool {
        true
    }
}

fn main() {
    common::terminal::run(Pegs);
}

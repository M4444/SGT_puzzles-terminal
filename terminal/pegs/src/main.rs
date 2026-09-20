mod pegs;
mod render;

use common::canvas::Coord;
use common::ffi::Preset;
use common::menu::{Menu, MenuState, TabSpec};
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

    fn render(
        &self,
        puzzle: &PegsPuzzle,
        wants_status_bar: bool,
        menu_state: MenuState,
        menu: &mut Menu<PegsAction>,
        terminal_columns: usize,
    ) -> String {
        render::render_game(puzzle, wants_status_bar, menu_state, menu, terminal_columns)
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

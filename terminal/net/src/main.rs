mod ffi;
mod input;
mod net;
mod render;

use common::canvas::{Canvas, Coord};
use common::ffi::Preset;
use common::menu::TabSpec;
use common::session::Session;
use common::terminal::TerminalGame;
use net::{NetAction, NetPuzzle};
use std::ffi::c_int;

/// Net, with the styles its board is drawn in.
#[derive(Default)]
struct Net {
    styles: render::Styles,
}

impl TerminalGame for Net {
    type Puzzle = NetPuzzle;
    type Action = NetAction;

    fn tab_specs(presets: &[Preset], can_solve: bool) -> Vec<TabSpec<NetAction>> {
        net::tab_specs(presets, can_solve)
    }

    fn draw_board(&self, canvas: &mut Canvas, puzzle: &NetPuzzle) {
        render::draw_board(canvas, puzzle, self.styles);
    }

    fn status(puzzle: &NetPuzzle) -> &str {
        &puzzle.status
    }

    fn is_action_current(&self, action: NetAction) -> bool {
        net::is_current(action, self.styles)
    }

    fn take_action(&mut self, action: NetAction) {
        input::take_action(action, &mut self.styles);
    }

    fn click_board(session: &mut Session<NetPuzzle>, position: Coord, button: c_int) -> bool {
        input::click_board(session, position, button)
    }
}

fn main() {
    common::terminal::run(Net::default());
}

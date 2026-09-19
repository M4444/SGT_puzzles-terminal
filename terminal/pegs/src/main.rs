mod pegs;
mod render;

use common::menu;
use common::session::Session;
use pegs::{PegsAction, PegsPuzzle};

/// The state Pegs' loop draws and updates.
struct Game {
    session: Session<PegsPuzzle>,
    menu: menu::Menu<PegsAction>,
}

fn main() {
    let session = pegs::new_session();
    let menu = menu::Menu::new(pegs::tab_specs(&session.presets(), session.can_solve()));
    let mut game = Game { session, menu };

    common::terminal::run(
        &mut game,
        |game, columns| {
            render::render_game(
                game.session.puzzle(),
                game.session.wants_status_bar(),
                menu::MenuState::new(&game.session),
                &mut game.menu,
                columns,
            )
        },
        |game, event| {
            common::input::take_event(
                event,
                &mut game.menu,
                &mut game.session,
                |action| match action {},
                // Clicks on the board do nothing.
                |_, _, _| true,
            )
        },
    );
}

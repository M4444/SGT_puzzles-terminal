mod ffi;
mod input;
mod net;
mod render;

use common::menu;
use common::session::Session;
use net::{NetAction, NetPuzzle};

/// The state Net's loop draws and updates.
struct Game {
    session: Session<NetPuzzle>,
    menu: menu::Menu<NetAction>,
    styles: render::Styles,
}

fn main() {
    let session = net::new_session();
    let menu = menu::Menu::new(net::tab_specs(&session.presets(), session.can_solve()));
    let mut game = Game { session, menu, styles: render::Styles::default() };

    common::terminal::run(
        &mut game,
        |game, columns| {
            render::render_game(
                game.session.puzzle(),
                game.session.wants_status_bar(),
                game.styles,
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
                |action| input::take_action(action, &mut game.styles),
                input::click_board,
            )
        },
    );
}

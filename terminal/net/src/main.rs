mod ffi;
mod input;
mod menu;
mod net;
mod render;

use common::session::Session;
use net::NetPuzzle;

/// The state Net's loop draws and updates.
struct Game {
    session: Session<NetPuzzle>,
    menu: menu::Menu,
    styles: render::Styles,
}

fn main() {
    let session = net::new_session();
    let menu = menu::Menu::new(&session.presets());
    let mut game = Game { session, menu, styles: render::Styles::default() };

    common::terminal::run(
        &mut game,
        |game, columns| {
            let menu_state = menu::MenuState {
                styles: game.styles,
                preset: game.session.which_preset(),
                can_undo: game.session.can_undo(),
                can_redo: game.session.can_redo(),
            };

            render::render_game(
                game.session.puzzle(),
                game.session.wants_status_bar(),
                menu_state,
                &mut game.menu,
                columns,
            )
        },
        |game, event| input::take_event(event, &mut game.menu, &mut game.session, &mut game.styles),
    );
}

mod pegs;
mod render;

use common::input::key_code;
use crossterm::event::Event;

fn main() {
    let mut session = pegs::new_session();

    common::terminal::run(
        &mut session,
        |session, columns| {
            render::render_game(session.puzzle(), session.wants_status_bar(), columns)
        },
        |session, event| match event {
            Event::Key(key) => match key_code(key) {
                Some(button) => session.process_key(button),
                None => true,
            },
            _ => true,
        },
    );
}

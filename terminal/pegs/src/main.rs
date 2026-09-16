mod pegs;
mod render;

use crossterm::event::{Event, KeyCode};

fn main() {
    let mut session = pegs::new_session();

    common::terminal::run(
        &mut session,
        |session, columns| {
            render::render_game(session.puzzle(), session.wants_status_bar(), columns)
        },
        |_session, event| !matches!(event, Event::Key(key) if key.code == KeyCode::Char('q')),
    );
}

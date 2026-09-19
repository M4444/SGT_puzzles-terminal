//! Setting the styles Net's menu offers and sending board clicks to the
//! tile they land on.

use crate::net::{self, NetAction, NetPuzzle};
use crate::render::Styles;
use common::board::{TileCoord, tiles_at};
use common::canvas::Coord;
use common::input::MIDDLE_BUTTON;
use common::session::Session;
use std::ffi::c_int;

/// Carries out one of Net's own menu actions.
pub(crate) fn take_action(action: NetAction, styles: &mut Styles) {
    match action {
        NetAction::SetCursorStyle(style) => styles.cursor = style,
        NetAction::SetLockStyle(style) => styles.lock = style,
    }
}

/// Takes a click the menu didn't claim and sends it to the tile it
/// resolves to, if any. Returns `false` if it signalled quit.
pub(crate) fn click_board(
    session: &mut Session<NetPuzzle>,
    position: Coord,
    button: c_int,
) -> bool {
    let possible_tiles = tiles_at(position, session.puzzle().dimensions);
    match resolve_tile(&possible_tiles, button, session) {
        Some(tile) => net::click_tile(session, tile, button),
        None => true,
    }
}

/// A click resolves to one tile either because it landed cleanly on
/// one, or (for anything but the lock-toggle button) because exactly
/// one of several overlapping candidates isn't locked, since a locked
/// tile can never be rotated.
fn resolve_tile(
    possible_tiles: &[TileCoord],
    button: c_int,
    session: &Session<NetPuzzle>,
) -> Option<TileCoord> {
    if let [only] = possible_tiles {
        Some(*only)
    } else if button != MIDDLE_BUTTON {
        if let [only] = net::exclude_locked(session, possible_tiles).as_slice() {
            Some(*only)
        } else {
            None
        }
    } else {
        None
    }
}

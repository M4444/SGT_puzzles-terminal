//! Sending board clicks to the tile they land on.

use crate::fifteen::{self, FifteenPuzzle};
use common::board::{Puzzle, tiles_at};
use common::canvas::Coord;
use common::session::Session;
use std::ffi::c_int;

/// Sends a click to the tile it lands on, when that tile could slide.
/// On a border shared between tiles it goes to the one tile there that
/// could slide and does nothing when more than one could. Returns
/// `false` if it signalled quit.
pub(crate) fn click_board(
    session: &mut Session<FifteenPuzzle>,
    position: Coord,
    button: c_int,
) -> bool {
    let possible_tiles = tiles_at(position, session.puzzle().dimensions());

    match fifteen::exclude_immovable(session.puzzle(), &possible_tiles).as_slice() {
        [tile] => fifteen::click_tile(session, *tile, button),
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::board::{left_side, right_side};
    use common::input::LEFT_BUTTON;

    /// A click on the border between the gap and the tile beside it can
    /// only mean that tile, since the gap has nothing to slide.
    #[test]
    fn clicking_the_gap_border_slides_the_tile_beside_it() {
        let mut session = Session::<FifteenPuzzle>::start();
        let gap = fifteen::gap_position(session.puzzle());
        let (gap_x, gap_y) = gap;
        let (border, tile) = if gap_x > 0 {
            (left_side(gap), (gap_x - 1, gap_y))
        } else {
            (right_side(gap), (gap_x + 1, gap_y))
        };

        click_board(&mut session, border, LEFT_BUTTON);

        assert_eq!(fifteen::gap_position(session.puzzle()), tile);
    }
}

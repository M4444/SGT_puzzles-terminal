//! The side menu: box-drawn buttons beside the board, separated
//! from it by a divider.

use crate::render::{draw_line, draw_rect_outline, Canvas, Coord, Rect, Size, Weight};

/// The menu's buttons, and what the game allows right now.
pub struct Menu {
    buttons: Vec<Button>,
    state: MenuState,
}

impl Menu {
    /// Places the menu beside the board, its buttons left to right.
    /// Drawing, sizing and hit-testing all read the rectangles fixed
    /// here, so they can't disagree about where a button sits.
    pub(crate) fn new(board_top_right: Coord, state: MenuState) -> Menu {
        let start = Coord::new(board_top_right.x + 1 + MENU_MARGIN, board_top_right.y);
        let mut x = start.x;
        let buttons = COMMON_ACTIONS
            .iter()
            .map(|entry| {
                let size = Size::new(button_width(entry.label), BUTTON_HEIGHT);
                let rect = Rect::new(Coord::new(x, start.y), size);
                x += size.width;
                Button { action: entry.action, label: entry.label, rect }
            })
            .collect();
        Menu { buttons, state }
    }

    /// Draws the divider, then the buttons.
    pub(crate) fn draw(&self, canvas: &mut Canvas) {
        draw_divider(canvas);
        for button in &self.buttons {
            draw_button(canvas, button, self.state);
        }
    }

    /// Which action, if any, a screen coordinate falls on. A dimmed
    /// button still answers, since the mid-end ignores an action it
    /// can't take.
    pub(crate) fn action_at(&self, position: impl Into<Coord>) -> Option<Action> {
        let position = position.into();
        self.buttons
            .iter()
            .find(|button| button.rect.contains(position))
            .map(|button| button.action)
    }

    /// The space the menu adds beside the board: its margin and its
    /// buttons.
    pub(crate) fn size(&self) -> Size {
        let width: usize = self
            .buttons
            .iter()
            .map(|button| button.rect.size.width)
            .sum();
        Size::new(MENU_MARGIN + width, BUTTON_HEIGHT)
    }
}

/// What a button does.
#[derive(Clone, Copy, Debug)]
pub enum Action {
    NewGame,
    Restart,
    Undo,
    Redo,
    Solve,
    Quit,
}

/// One entry of the menu, before `Menu::new` places it.
#[derive(Clone, Copy, Debug)]
struct Entry {
    action: Action,
    label: &'static str,
}

/// One button of the menu, with the rectangle it occupies on the
/// canvas.
#[derive(Clone, Copy, Debug)]
struct Button {
    action: Action,
    label: &'static str,
    rect: Rect,
}

/// The actions every game's menu offers.
const COMMON_ACTIONS: &[Entry] = &[
    Entry { action: Action::NewGame, label: "New Game" },
    Entry { action: Action::Restart, label: "Restart" },
    Entry { action: Action::Undo, label: "Undo" },
    Entry { action: Action::Redo, label: "Redo" },
    Entry { action: Action::Solve, label: "Solve" },
    Entry { action: Action::Quit, label: "Quit" },
];

/// Blank columns on each side of the divider, keeping it clear of
/// the frame's own border.
const DIVIDER_MARGIN: usize = 1;

/// Columns between the board and the menu: the divider itself,
/// with its margin either side.
const MENU_MARGIN: usize = 2 * DIVIDER_MARGIN + 1;

/// A box-drawn button's on-screen height: a border row above and
/// below the row holding the label.
const BUTTON_HEIGHT: usize = 3;

/// A box-drawn button's on-screen width: the label, a one-column
/// margin either side of it, and the button's own two border columns.
fn button_width(label: &str) -> usize {
    label.chars().count() + 4
}

/// Draws a light vertical line between the frame and the menu,
/// spanning the frame's own height.
fn draw_divider(canvas: &mut Canvas) {
    let x = canvas.frame.right() + 1 + DIVIDER_MARGIN;
    let (top, bottom) = (canvas.frame.top(), canvas.frame.bottom());
    draw_line(canvas, (x, top), (x, bottom), Weight::Light);
}

/// Draws one button in its own rectangle, dimmed if its action can't
/// be taken.
fn draw_button(canvas: &mut Canvas, button: &Button, state: MenuState) {
    let rect = button.rect;
    draw_rect_outline(canvas, rect, Weight::Light);
    canvas.draw_text((rect.left() + 1, rect.top() + 1), &format!(" {} ", button.label));
    if !available(button.action, state) {
        canvas.mark_dimmed_region(rect.coords());
    }
}

/// What the menu needs from the game to draw itself: the actions whose
/// availability depends on the move history.
#[derive(Clone, Copy, Debug)]
pub struct MenuState {
    pub can_undo: bool,
    pub can_redo: bool,
}

/// Whether an action can be taken right now. Every action but Undo and
/// Redo is always available.
fn available(action: Action, state: MenuState) -> bool {
    match action {
        Action::Undo => state.can_undo,
        Action::Redo => state.can_redo,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Placed against a 5x5 board's frame, with nothing dimmed.
    fn placed() -> Menu {
        let state = MenuState { can_undo: true, can_redo: true };
        Menu::new(crate::render::grid_frame((5, 5)).top_right(), state)
    }

    #[test]
    fn menu_width_sums_its_buttons_and_the_margin() {
        // Each button is its label plus a margin either side and two
        // border columns, and the divider adds its own three.
        assert_eq!(placed().size().width, 59);
    }

    #[test]
    fn menu_height_is_one_button() {
        // A border row above and below the row holding the label.
        assert_eq!(placed().size().height, 3);
    }

    fn unavailable_labels(state: MenuState) -> Vec<&'static str> {
        COMMON_ACTIONS
            .iter()
            .filter(|entry| !available(entry.action, state))
            .map(|entry| entry.label)
            .collect()
    }

    #[test]
    fn only_undo_and_redo_follow_the_game_state() {
        let neither = MenuState { can_undo: false, can_redo: false };
        assert_eq!(unavailable_labels(neither), ["Undo", "Redo"]);

        let both = MenuState { can_undo: true, can_redo: true };
        assert!(unavailable_labels(both).is_empty());
    }

    #[test]
    fn undo_and_redo_are_independent() {
        let undo_only = MenuState { can_undo: true, can_redo: false };
        assert_eq!(unavailable_labels(undo_only), ["Redo"]);
    }

    /// Where the buttons are placed is where clicks find them. On a
    /// 5x5 board the frame ends at column 24, so the row starts at 28
    /// and the six buttons run to column 83.
    #[test]
    fn a_click_lands_on_the_button_it_is_over() {
        let menu = placed();
        let find = |x, y| menu.action_at((x, y));

        assert!(matches!(find(28, 0), Some(Action::NewGame)));
        assert!(matches!(find(39, 0), Some(Action::NewGame)));
        assert!(matches!(find(40, 0), Some(Action::Restart)));
        assert!(matches!(find(83, 2), Some(Action::Quit)));

        // Left of the row, past its end, and below its last row.
        assert!(find(27, 0).is_none());
        assert!(find(84, 0).is_none());
        assert!(find(28, 3).is_none());
    }
}

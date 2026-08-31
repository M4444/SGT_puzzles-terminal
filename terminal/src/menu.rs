//! The side menu: box-drawn buttons beside the board, separated
//! from it by a divider.

use crate::render::{draw_line, draw_rect_outline, Canvas, Coord, Rect, Size, Weight};

/// The menu's buttons.
pub struct Menu {
    buttons: Vec<Button>,
}

impl Menu {
    /// Places the menu beside the board, its buttons left to right.
    /// Drawing and sizing both read the rectangles fixed here, so they
    /// can't disagree about where a button sits.
    pub(crate) fn new(board_top_right: Coord) -> Menu {
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
        Menu { buttons }
    }

    /// Draws the divider, then the buttons.
    pub(crate) fn draw(&self, canvas: &mut Canvas) {
        draw_divider(canvas);
        for button in &self.buttons {
            draw_button(canvas, button);
        }
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
enum Action {
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
fn draw_button(canvas: &mut Canvas, button: &Button) {
    let rect = button.rect;
    draw_rect_outline(canvas, rect, Weight::Light);
    canvas.draw_text((rect.left() + 1, rect.top() + 1), &format!(" {} ", button.label));
    if !available(button.action) {
        canvas.mark_dimmed_region(rect.coords());
    }
}

/// Whether an action can be taken right now. Undo and Redo are the
/// only ones that ever vary, and both are unavailable on a freshly
/// generated game.
fn available(action: Action) -> bool {
    !matches!(action, Action::Undo | Action::Redo)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Placed against a 5x5 board's frame.
    fn placed() -> Menu {
        Menu::new(crate::render::grid_frame((5, 5)).top_right())
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

    #[test]
    fn only_undo_and_redo_are_unavailable() {
        let unavailable: Vec<&str> = COMMON_ACTIONS
            .iter()
            .filter(|entry| !available(entry.action))
            .map(|entry| entry.label)
            .collect();

        assert_eq!(unavailable, ["Undo", "Redo"]);
    }
}

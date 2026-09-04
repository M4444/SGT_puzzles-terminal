//! The side menu: the tabs beside the board, separated from it by a
//! divider.

use crate::render::{draw_line, draw_rect_outline, Canvas, Coord, Rect, Size, Weight};

/// The menu's tabs, listed top to bottom.
pub struct Menu {
    tabs: Vec<Tab>,
    /// Where the tabs start after the divider.
    content_top_left: Coord,
    size: Size,
}

impl Menu {
    /// Builds the menu beside the board, every named tab closed.
    pub(crate) fn new(board_top_right: Coord) -> Menu {
        let tabs = TABS
            .iter()
            .map(|spec| Tab { spec, open: false, header: None, body: None })
            .collect();
        let content_top_left = Coord::new(board_top_right.x + 1 + MENU_MARGIN, board_top_right.y);
        let mut menu = Menu { tabs, content_top_left, size: Size::new(0, 0) };
        menu.refresh_tabs();
        menu
    }

    /// Refreshes each tab's header and any body it's showing, top to
    /// bottom, and calculates the menu's size from that.
    fn refresh_tabs(&mut self) {
        let width = TABS.iter().map(TabSpec::width).max().unwrap_or(0);
        let left = self.content_top_left.x;
        let top = self.content_top_left.y;
        let mut row = top;
        for tab in &mut self.tabs {
            tab.header = tab.spec.name.map(|name| {
                let rect = Rect::row(Coord::new(left, row), width);
                row += 1;
                Header { rect, name }
            });
            tab.body = (tab.spec.name.is_none() || tab.open).then(|| {
                let body = tab.spec.body.place(Coord::new(left, row));
                row += body.height();
                body
            });
        }
        self.size = Size::new(MENU_MARGIN + width, row - top);
    }

    /// Draws the divider, then each tab's header and any body it's
    /// showing.
    pub(crate) fn draw(&self, canvas: &mut Canvas, availability: ActionAvailability) {
        draw_divider(canvas);
        for tab in &self.tabs {
            if let Some(header) = &tab.header {
                draw_header(canvas, header, tab.open);
            }
            match &tab.body {
                Some(Body::Buttons(buttons)) => {
                    for button in buttons {
                        draw_button(canvas, button, availability);
                    }
                }
                Some(Body::Legend(legend)) => draw_legend(canvas, legend),
                None => {}
            }
        }
    }

    /// Takes a click. A header opens or closes its tab and answers
    /// nothing; a button answers its action, dimmed or not, since the
    /// mid-end ignores an action it can't take.
    pub(crate) fn click(&mut self, position: impl Into<Coord>) -> Option<Action> {
        let position = position.into();
        let header_hit = self.tabs.iter().position(|tab| {
            tab.header
                .as_ref()
                .is_some_and(|header| header.rect.contains(position))
        });
        if let Some(index) = header_hit {
            self.tabs[index].open = !self.tabs[index].open;
            self.refresh_tabs();
            return None;
        }
        for tab in &self.tabs {
            if let Some(Body::Buttons(buttons)) = &tab.body {
                if let Some(button) = buttons.iter().find(|button| button.rect.contains(position)) {
                    return Some(button.spec.action);
                }
            }
        }
        None
    }

    /// The space the menu adds beside the board.
    pub(crate) fn size(&self) -> Size {
        self.size
    }
}

/// A tab's name and body. A named tab draws a header and can be
/// collapsed; an unnamed one is always just its body.
struct TabSpec {
    name: Option<&'static str>,
    body: BodySpec,
}

/// What a tab shows.
enum BodySpec {
    Buttons(&'static [ButtonSpec]),
    Legend(&'static [LegendGroup]),
}

/// A tab's header and body. An unnamed tab has no header; a closed
/// one has no body.
struct Tab {
    spec: &'static TabSpec,
    open: bool,
    header: Option<Header>,
    body: Option<Body>,
}

/// A tab's header: a line across the menu with its name written over
/// it.
struct Header {
    rect: Rect,
    name: &'static str,
}

/// A tab's body.
enum Body {
    Buttons(Vec<Button>),
    Legend(Legend),
}

impl TabSpec {
    /// The columns this tab needs: its body, or its header if the name
    /// is the wider of the two. A header spans the offset, the name
    /// with a space either side, and the arrow's own column.
    fn width(&self) -> usize {
        let header = self
            .name
            .map_or(0, |name| HEADER_NAME_OFFSET + name.chars().count() + 3);
        self.body.width().max(header)
    }
}

impl BodySpec {
    fn width(&self) -> usize {
        match self {
            BodySpec::Buttons(specs) => specs.iter().map(|spec| button_width(spec.label)).sum(),
            BodySpec::Legend(groups) => legend_width(groups),
        }
    }

    fn place(&self, top_left: Coord) -> Body {
        match self {
            BodySpec::Buttons(specs) => {
                let mut x = top_left.x;
                let buttons = specs
                    .iter()
                    .map(|spec| {
                        let size = Size::new(button_width(spec.label), BUTTON_HEIGHT);
                        let rect = Rect::new(Coord::new(x, top_left.y), size);
                        x += size.width;
                        Button { spec, rect }
                    })
                    .collect();
                Body::Buttons(buttons)
            }
            BodySpec::Legend(groups) => Body::Legend(Legend { top_left, groups }),
        }
    }
}

impl Body {
    fn height(&self) -> usize {
        match self {
            Body::Buttons(_) => BUTTON_HEIGHT,
            Body::Legend(legend) => legend_height(legend.groups),
        }
    }
}

/// A legend's rows and where they start. Nothing in it is clickable,
/// so a corner to draw from is all it needs.
struct Legend {
    groups: &'static [LegendGroup],
    top_left: Coord,
}

/// An input and what it does.
pub(crate) struct LegendEntry {
    pub(crate) input: &'static str,
    pub(crate) description: &'static str,
}

/// A group of legend rows, under an optional label.
pub(crate) struct LegendGroup {
    pub(crate) label: Option<&'static str>,
    pub(crate) entries: &'static [LegendEntry],
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

/// A button's action and label.
#[derive(Clone, Copy, Debug)]
struct ButtonSpec {
    action: Action,
    label: &'static str,
}

/// One button of the menu, with the rectangle it occupies.
#[derive(Clone, Copy, Debug)]
struct Button {
    spec: &'static ButtonSpec,
    rect: Rect,
}

/// The actions every game's menu offers.
const COMMON_ACTIONS: &[ButtonSpec] = &[
    ButtonSpec { action: Action::NewGame, label: "New Game" },
    ButtonSpec { action: Action::Restart, label: "Restart" },
    ButtonSpec { action: Action::Undo, label: "Undo" },
    ButtonSpec { action: Action::Redo, label: "Redo" },
    ButtonSpec { action: Action::Solve, label: "Solve" },
    ButtonSpec { action: Action::Quit, label: "Quit" },
];

/// The menu's tabs, in the order they appear.
const TABS: &[TabSpec] = &[
    TabSpec { name: None, body: BodySpec::Buttons(COMMON_ACTIONS) },
    TabSpec { name: Some("Game Controls"), body: BodySpec::Legend(crate::net::GAME_CONTROLS) },
];

/// Columns of line before a header's padded name starts, so the name
/// reads as sitting on the line.
const HEADER_NAME_OFFSET: usize = 2;

/// Between a legend row's input and its description.
const LEGEND_SEPARATOR: &str = " - ";

fn widest_legend_input(groups: &[LegendGroup]) -> usize {
    groups
        .iter()
        .flat_map(|group| group.entries)
        .map(|entry| entry.input.chars().count())
        .max()
        .unwrap_or(0)
}

/// The widest row a legend draws.
fn legend_width(groups: &[LegendGroup]) -> usize {
    let label_widths = groups
        .iter()
        .filter_map(|group| group.label)
        .map(|label| label.chars().count());
    let input_width = widest_legend_input(groups);
    let entry_widths = groups.iter().flat_map(|group| group.entries).map(|entry| {
        input_width + LEGEND_SEPARATOR.chars().count() + entry.description.chars().count()
    });
    label_widths.chain(entry_widths).max().unwrap_or(0)
}

/// The rows a legend draws: each group's label and entries, with a
/// blank row between groups.
fn legend_height(groups: &[LegendGroup]) -> usize {
    let filled_rows: usize = groups
        .iter()
        .map(|group| usize::from(group.label.is_some()) + group.entries.len())
        .sum();
    let blank_rows = groups.len().saturating_sub(1);
    filled_rows + blank_rows
}

/// Blank columns on each side of the divider, keeping it clear of
/// the frame's own border.
const DIVIDER_MARGIN: usize = 1;

/// Columns between the board and the menu: the divider itself,
/// with its margin either side.
const MENU_MARGIN: usize = 2 * DIVIDER_MARGIN + 1;

/// A box-drawn button's height: a border row above and below the row
/// holding the label.
const BUTTON_HEIGHT: usize = 3;

/// A box-drawn button's width: the label, a one-column margin either
/// side of it, and the button's own two border columns.
fn button_width(label: &str) -> usize {
    label.chars().count() + 4
}

/// Draws a light vertical line between the board and the menu,
/// spanning the board's full height.
fn draw_divider(canvas: &mut Canvas) {
    let board = canvas.board();
    let column = board.right() + 1 + DIVIDER_MARGIN;
    draw_line(canvas, (column, board.top()), (column, board.bottom()), Weight::Light);
}

/// Draws a header: a line across the menu, the tab's name written
/// over it, and an arrow at the right end showing which way a click
/// will take it.
fn draw_header(canvas: &mut Canvas, header: &Header, open: bool) {
    let rect = header.rect;
    let row = rect.top();
    draw_line(canvas, (rect.left(), row), (rect.right(), row), Weight::Light);

    canvas.draw_text((rect.left() + HEADER_NAME_OFFSET, row), &format!(" {} ", header.name));
    canvas.draw_text((rect.right(), row), if open { "▲" } else { "▼" });
}

/// Draws a legend's groups from its top left, each label above its own
/// entries and a blank row between groups.
fn draw_legend(canvas: &mut Canvas, legend: &Legend) {
    let input_width = widest_legend_input(legend.groups);
    let start_column = legend.top_left.x;
    let mut row = legend.top_left.y;
    for (index, group) in legend.groups.iter().enumerate() {
        // Add a blank row above every group but the first.
        if index > 0 {
            row += 1;
        }
        if let Some(label) = group.label {
            canvas.draw_text((start_column, row), label);
            let cells = Rect::row(Coord::new(start_column, row), label.chars().count());
            canvas.mark_reversed_region(cells.coords());
            row += 1;
        }
        for entry in group.entries {
            let input = format!("{:<input_width$}", entry.input);
            let text = format!("{input}{LEGEND_SEPARATOR}{}", entry.description);
            canvas.draw_text((start_column, row), &text);
            row += 1;
        }
    }
}

/// Draws one button in its own rectangle, dimmed if its action can't
/// be taken.
fn draw_button(canvas: &mut Canvas, button: &Button, availability: ActionAvailability) {
    let rect = button.rect;
    draw_rect_outline(canvas, rect, Weight::Light);
    canvas.draw_text((rect.left() + 1, rect.top() + 1), &format!(" {} ", button.spec.label));
    if !available(button.spec.action, availability) {
        canvas.mark_dimmed_region(rect.coords());
    }
}

/// What the menu needs from the game to draw itself: the actions whose
/// availability depends on the move history.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ActionAvailability {
    pub can_undo: bool,
    pub can_redo: bool,
}

/// Whether an action can be taken right now. Every action but Undo and
/// Redo is always available.
fn available(action: Action, availability: ActionAvailability) -> bool {
    match action {
        Action::Undo => availability.can_undo,
        Action::Redo => availability.can_redo,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A menu beside a 5x5 board, every tab closed.
    fn menu() -> Menu {
        Menu::new(crate::render::grid_frame((5, 5)).top_right())
    }

    /// The widest tab sets the width, open or not, so opening one
    /// never shifts the menu sideways. The legend's longest row is
    /// 57 columns against the button row's 56.
    #[test]
    fn menu_width_is_its_widest_tab_and_the_margin() {
        assert_eq!(menu().size().width, 60);
    }

    /// A closed tab shows its header and nothing else.
    #[test]
    fn a_closed_tab_costs_only_its_header() {
        // The button row's three, and one for Game Controls' header.
        assert_eq!(menu().size().height, 4);
    }

    /// Clicking a header opens the tab, which answers no action and
    /// makes room for the body.
    #[test]
    fn clicking_a_header_opens_the_tab() {
        let mut menu = menu();
        // Game Controls' header, on the row below the button row.
        assert!(menu.click((28, 3)).is_none());

        // Mouse has a label and three entries, Keyboard a label and
        // nine, with a blank row between the groups.
        assert_eq!(menu.size().height, 4 + 15);

        assert!(menu.click((28, 3)).is_none());
        assert_eq!(menu.size().height, 4);
    }

    fn unavailable_labels(availability: ActionAvailability) -> Vec<&'static str> {
        COMMON_ACTIONS
            .iter()
            .filter(|spec| !available(spec.action, availability))
            .map(|spec| spec.label)
            .collect()
    }

    #[test]
    fn only_undo_and_redo_can_be_unavailable() {
        let neither = ActionAvailability { can_undo: false, can_redo: false };
        assert_eq!(unavailable_labels(neither), ["Undo", "Redo"]);

        let both = ActionAvailability { can_undo: true, can_redo: true };
        assert!(unavailable_labels(both).is_empty());
    }

    #[test]
    fn undo_and_redo_are_independent() {
        let undo_only = ActionAvailability { can_undo: true, can_redo: false };
        assert_eq!(unavailable_labels(undo_only), ["Redo"]);
    }

    /// Where the buttons are placed is where clicks find them. On a
    /// 5x5 board the frame ends at column 24, so the row starts at 28
    /// and the six buttons run to column 83.
    #[test]
    fn a_click_lands_on_the_button_it_is_over() {
        let mut menu = menu();
        let mut find = |x, y| menu.click((x, y));

        assert!(matches!(find(28, 0), Some(Action::NewGame)));
        assert!(matches!(find(39, 0), Some(Action::NewGame)));
        assert!(matches!(find(40, 0), Some(Action::Restart)));
        assert!(matches!(find(83, 2), Some(Action::Quit)));

        // Left of the row, past its end, and below its last row.
        assert!(find(27, 0).is_none());
        assert!(find(84, 0).is_none());
        assert!(find(28, 4).is_none());
    }
}

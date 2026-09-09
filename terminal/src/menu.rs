//! The side menu: the tabs beside the board, separated from it by a
//! divider.

use crate::render::{
    draw_line, draw_rect_outline, Canvas, Coord, CursorStyle, LockStyle, Mark, Rect, Size, Styles,
    Weight,
};

/// The menu's tabs, listed top to bottom.
pub struct Menu {
    tabs: Vec<Tab>,
    /// Where the tabs start after the divider.
    content_top_left: Coord,
    size: Size,
    /// Which tab has focus. `None` leaves it on the board.
    focus: Option<usize>,
}

impl Menu {
    /// Builds the menu beside the board, every named tab closed.
    pub(crate) fn new(board_top_right: Coord) -> Menu {
        let tabs = TABS
            .iter()
            .map(|spec| Tab { spec, open: false, cursor: 0, header: None, body: None })
            .collect();
        let content_top_left = Coord::new(board_top_right.x + 1 + MENU_MARGIN, board_top_right.y);
        let mut menu = Menu { tabs, content_top_left, size: Size::new(0, 0), focus: None };
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
    pub(crate) fn draw(
        &self,
        canvas: &mut Canvas,
        availability: ActionAvailability,
        styles: Styles,
    ) {
        draw_divider(canvas);
        for (index, tab) in self.tabs.iter().enumerate() {
            let focused = self.focus == Some(index);
            if let Some(header) = &tab.header {
                draw_header(canvas, header, tab.open, focused);
            }
            match &tab.body {
                Some(Body::Buttons(buttons)) => {
                    for (position, button) in buttons.iter().enumerate() {
                        let has_cursor = focused && position == tab.cursor;
                        draw_button(canvas, button, availability, has_cursor, styles);
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

    /// Whether focus is on the menu rather than the board.
    pub(crate) fn has_focus(&self) -> bool {
        self.focus.is_some()
    }

    /// Returns focus to the board.
    pub(crate) fn clear_focus(&mut self) {
        self.focus = None;
    }

    /// Moves focus on to the next tab, returning it to the board once
    /// it runs off the last one.
    pub(crate) fn focus_next(&mut self) {
        self.focus = match self.focus {
            None => Some(0),
            Some(LAST_TAB) => None,
            Some(index) => Some(index + 1),
        };
    }

    /// The reverse of `focus_next`.
    pub(crate) fn focus_previous(&mut self) {
        self.focus = match self.focus {
            None => Some(LAST_TAB),
            Some(0) => None,
            Some(index) => Some(index - 1),
        };
    }

    pub(crate) fn cursor_left(&mut self) {
        if let Some(tab) = self.focused_tab() {
            tab.cursor_previous();
        }
    }

    pub(crate) fn cursor_right(&mut self) {
        if let Some(tab) = self.focused_tab() {
            tab.cursor_next();
        }
    }

    fn focused_tab(&mut self) -> Option<&mut Tab> {
        Some(&mut self.tabs[self.focus?])
    }

    /// Takes Enter on the focused tab. A tab showing buttons answers
    /// the action of the button its cursor is on. Otherwise the tab
    /// opens or closes, as clicking its header would.
    pub(crate) fn press(&mut self) -> Option<Action> {
        let index = self.focus?;
        let tab = &self.tabs[index];
        if let Some(Body::Buttons(buttons)) = &tab.body {
            return Some(buttons[tab.cursor].spec.action);
        }

        if tab.header.is_some() {
            self.tabs[index].open = !tab.open;
            self.refresh_tabs();
        }
        None
    }
}

/// A tab's name and body. A named tab draws a header and can be
/// collapsed; an unnamed one is always just its body.
struct TabSpec {
    name: Option<&'static str>,
    body: BodySpec,
}

/// What a tab shows. Choices are buttons standing for one setting's
/// options, so each carries a tick column showing which is in use.
enum BodySpec {
    Buttons(&'static [ButtonSpec]),
    Choices(&'static [ButtonSpec]),
    Legend(&'static [LegendGroup]),
}

/// A tab's header and body. An unnamed tab has no header; a closed
/// one has no body.
struct Tab {
    spec: &'static TabSpec,
    open: bool,
    /// Where the cursor sits among the tab's buttons.
    cursor: usize,
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

impl Tab {
    /// Moves the button cursor on by one, stopping at the last button.
    fn cursor_next(&mut self) {
        if let Some(Body::Buttons(buttons)) = &self.body {
            self.cursor = (self.cursor + 1).min(buttons.len().saturating_sub(1));
        }
    }

    /// Moves the button cursor back by one, stopping at the first.
    fn cursor_previous(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }
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
            BodySpec::Buttons(specs) => buttons_width(specs, false),
            BodySpec::Choices(specs) => buttons_width(specs, true),
            BodySpec::Legend(groups) => legend_width(groups),
        }
    }

    fn place(&self, top_left: Coord) -> Body {
        match self {
            BodySpec::Buttons(specs) => Body::Buttons(place_buttons(specs, top_left, false)),
            BodySpec::Choices(specs) => Body::Buttons(place_buttons(specs, top_left, true)),
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
    SetCursorStyle(CursorStyle),
    SetLockStyle(LockStyle),
}

/// Shown in a choice button's tick column when it's the current one.
const TICK: char = '✓';

/// Whether a choice's action sets the style the board already uses.
fn is_current(action: Action, styles: Styles) -> bool {
    match action {
        Action::SetCursorStyle(style) => style == styles.cursor,
        Action::SetLockStyle(style) => style == styles.lock,
        _ => false,
    }
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
    /// Whether the button is one of a set of choices.
    choice: bool,
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

/// The keys the mid-end takes, which only reach it while the board has
/// focus.
const BOARD_SHORTCUTS: &[LegendEntry] = &[
    LegendEntry { input: "N", description: "new game" },
    LegendEntry { input: "U", description: "undo move" },
    LegendEntry { input: "R", description: "redo move" },
    LegendEntry { input: "Ctrl + S", description: "solve game" },
    LegendEntry { input: "Q", description: "quit game" },
];

/// The menu's own keys.
const MENU_KEYS: &[LegendEntry] = &[
    LegendEntry { input: "Tab", description: "move focus" },
    LegendEntry { input: "Shift + Tab", description: "move focus back" },
    LegendEntry { input: "Arrows", description: "move between buttons" },
    LegendEntry { input: "Enter / Left mouse button", description: "press a button or open a tab" },
];

const MENU_CONTROLS: &[LegendGroup] = &[
    LegendGroup { label: Some("Game:"), entries: BOARD_SHORTCUTS },
    LegendGroup { label: Some("Menu:"), entries: MENU_KEYS },
];

const CURSOR_STYLES: &[ButtonSpec] = &[
    ButtonSpec { action: Action::SetCursorStyle(CursorStyle::Outline), label: "Outline" },
    ButtonSpec { action: Action::SetCursorStyle(CursorStyle::ReverseTileCenter), label: "Center" },
    ButtonSpec { action: Action::SetCursorStyle(CursorStyle::ReverseTileFull), label: "Full" },
];

const LOCK_STYLES: &[ButtonSpec] = &[
    ButtonSpec { action: Action::SetLockStyle(LockStyle::ReverseTileConnected), label: "Merged" },
    ButtonSpec { action: Action::SetLockStyle(LockStyle::ReverseTileCenter), label: "Center" },
    ButtonSpec { action: Action::SetLockStyle(LockStyle::ReverseTileFull), label: "Full" },
];

/// The menu's tabs, in the order they appear.
const TABS: &[TabSpec] = &[
    TabSpec { name: None, body: BodySpec::Buttons(COMMON_ACTIONS) },
    TabSpec { name: Some("Cursor Style"), body: BodySpec::Choices(CURSOR_STYLES) },
    TabSpec { name: Some("Lock Style"), body: BodySpec::Choices(LOCK_STYLES) },
    TabSpec { name: Some("Menu Controls"), body: BodySpec::Legend(MENU_CONTROLS) },
    TabSpec { name: Some("Game Controls"), body: BodySpec::Legend(crate::net::GAME_CONTROLS) },
];

const LAST_TAB: usize = TABS.len() - 1;

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

/// Every button pads a margin and a border column either side and a
/// choice adds a tick with a space before its label.
fn button_width(spec: &ButtonSpec, choice: bool) -> usize {
    let tick = if choice { 2 } else { 0 };
    spec.label.chars().count() + tick + 4
}

fn buttons_width(specs: &[ButtonSpec], choice: bool) -> usize {
    specs.iter().map(|spec| button_width(spec, choice)).sum()
}

fn place_buttons(specs: &'static [ButtonSpec], top_left: Coord, choice: bool) -> Vec<Button> {
    let mut x = top_left.x;
    specs
        .iter()
        .map(|spec| {
            let size = Size::new(button_width(spec, choice), BUTTON_HEIGHT);
            let rect = Rect::new(Coord::new(x, top_left.y), size);
            x += size.width;
            Button { spec, rect, choice }
        })
        .collect()
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
/// will take it. The line is heavy while the tab has focus.
fn draw_header(canvas: &mut Canvas, header: &Header, open: bool, focused: bool) {
    let rect = header.rect;
    let row = rect.top();
    let weight = if focused {
        Weight::Heavy
    } else {
        Weight::Light
    };
    draw_line(canvas, (rect.left(), row), (rect.right(), row), weight);

    let name_start = Coord::new(rect.left() + HEADER_NAME_OFFSET, row);
    let name = format!(" {} ", header.name);
    canvas.draw_text_marked(name_start, &name, focused.then_some(Mark::Bold));
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
            canvas.draw_text_marked((start_column, row), label, Some(Mark::Reversed));
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

/// Draws one button in its own rectangle, ticked if it is the choice
/// in use, dimmed if its action can't be taken and outlined heavy
/// while the cursor is on it.
fn draw_button(
    canvas: &mut Canvas,
    button: &Button,
    availability: ActionAvailability,
    has_cursor: bool,
    styles: Styles,
) {
    let rect = button.rect;
    let weight = if has_cursor {
        Weight::Heavy
    } else {
        Weight::Light
    };
    draw_rect_outline(canvas, rect, weight);
    let label_start = Coord::new(rect.left() + 1, rect.top() + 1);
    let label = if button.choice {
        let tick = if is_current(button.spec.action, styles) {
            TICK
        } else {
            ' '
        };
        format!(" {tick} {} ", button.spec.label)
    } else {
        format!(" {} ", button.spec.label)
    };
    canvas.draw_text_marked(label_start, &label, has_cursor.then_some(Mark::Bold));
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
        // The button row's three, and a header each for the four
        // named tabs.
        assert_eq!(menu().size().height, 7);
    }

    /// Clicking a header opens the tab, which answers no action and
    /// makes room for the body.
    #[test]
    fn clicking_a_header_opens_the_tab() {
        let mut menu = menu();
        // Cursor Style's header, on the row below the button row.
        assert!(menu.click((28, 3)).is_none());

        // Its buttons take one row of three, like any button body.
        assert_eq!(menu.size().height, 7 + BUTTON_HEIGHT);

        assert!(menu.click((28, 3)).is_none());
        assert_eq!(menu.size().height, 7);
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

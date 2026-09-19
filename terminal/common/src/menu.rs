//! The side menu: the tabs beside the board, separated from it by a
//! divider.

use crate::canvas::{Canvas, Coord, Mark, Rect, Size, Weight, draw_line, draw_rect_outline};
use crate::ffi::Preset;
use crate::session::Session;

/// The menu's tabs, listed top to bottom, with `G` standing for the
/// game's own action type.
pub struct Menu<G> {
    tabs: Vec<Tab<G>>,
    size: Size,
    /// Where the menu sits on screen. Its headers, buttons and legends
    /// are laid out from (0, 0) and shifted by this position to be drawn
    /// or clicked.
    top_left: Coord,
    divider_length: usize,
    /// How wide the terminal is. Headers stop at its edge.
    terminal_columns: usize,
    /// What the menu's focus is on. When it's `None`, the board has it.
    focus: Option<Focus>,
}

/// What the focus can be on: a tab's header, which opens and closes it,
/// or the buttons of a tab showing them. A legend takes no focus, since
/// there is nothing in it to press.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Focus {
    Header(usize),
    Buttons(usize),
}

impl<G: Copy> Menu<G> {
    /// Builds the menu with every named tab closed, drawn at the top left
    /// of the screen until `set_placement` says where it goes.
    pub fn new(tab_specs: Vec<TabSpec<G>>) -> Menu<G> {
        let tabs = tab_specs
            .into_iter()
            .map(|spec| Tab { spec, open: false, cursor: 0, header: None, body: None })
            .collect();
        let mut menu = Menu {
            tabs,
            size: Size::new(0, 0),
            top_left: Coord::new(0, 0),
            divider_length: 0,
            terminal_columns: usize::MAX,
            focus: None,
        };

        menu.refresh_tabs();
        menu
    }

    /// Refreshes each tab's header and any body it's showing, top to
    /// bottom to the right of the divider, and calculates the menu's
    /// size from that.
    fn refresh_tabs(&mut self) {
        let width = self.tabs.iter().map(|tab| tab.spec.width()).max().unwrap_or(0);
        let mut row = 0;

        for tab in &mut self.tabs {
            tab.header = tab.spec.name.map(|name| {
                let rect = Rect::row(Coord::new(MENU_MARGIN, row), width);
                row += 1;
                Header { rect, name }
            });
            tab.body = (tab.spec.name.is_none() || tab.open).then(|| {
                let body = tab.spec.body.place(Coord::new(MENU_MARGIN, row));
                row += body.height();
                body
            });
        }

        self.size = Size::new(MENU_MARGIN + width, row);
    }

    /// Sets the menu's position just right of the board, level with its
    /// top, with its divider running the board's full height and its
    /// headers stopping at the terminal's edge.
    pub fn set_placement(&mut self, board_rect: Rect, terminal_columns: usize) {
        self.top_left = Coord::new(board_rect.right() + 1, board_rect.top());
        self.divider_length = board_rect.bottom() - board_rect.top() + 1;
        self.terminal_columns = terminal_columns;
    }

    /// Draws the divider, then each tab's header and any body it's
    /// showing. A choice the game already uses is ticked. For the
    /// game's own actions, `is_game_action_current` decides that.
    pub fn draw(
        &self,
        canvas: &mut Canvas,
        menu_state: MenuState,
        is_game_action_current: impl Fn(G) -> bool,
    ) {
        let offset = self.top_left;

        draw_divider(canvas, offset, self.divider_length);

        for (index, tab) in self.tabs.iter().enumerate() {
            if let Some(header) = &tab.header {
                let focused = self.focus == Some(Focus::Header(index));
                draw_header(canvas, header, offset, self.terminal_columns, tab.open, focused);
            }
            match &tab.body {
                Some(Body::Buttons(buttons)) => {
                    let focused = self.focus == Some(Focus::Buttons(index));
                    for (position, button) in buttons.iter().enumerate() {
                        let has_cursor = focused && position == tab.cursor;
                        let ticked = button.choice
                            && is_current(button.spec.action, menu_state, &is_game_action_current);
                        draw_button(canvas, button, offset, has_cursor, menu_state, ticked);
                    }
                }
                Some(Body::Legend(legend)) => draw_legend(canvas, legend, offset),
                None => {}
            }
        }
    }

    /// Takes a click at a screen position. A header opens or closes its
    /// tab and returns no action. A button returns its own, dimmed or
    /// not, since the mid-end ignores an action it can't take.
    pub fn click(&mut self, position: impl Into<Coord>) -> Option<Action<G>> {
        let position = position.into();
        // A click left of or above the menu can't land on it.
        let position = Coord::new(
            position.x.checked_sub(self.top_left.x)?,
            position.y.checked_sub(self.top_left.y)?,
        );
        let header_hit = self.tabs.iter().position(|tab| {
            tab.header
                .as_ref()
                .is_some_and(|header| header.rect.contains(position))
        });
        if let Some(index) = header_hit {
            self.tabs[index].toggle_open();
            self.refresh_tabs();
            return None;
        }

        for tab in &self.tabs {
            if let Some(buttons) = tab.shown_buttons()
                && let Some(button) = buttons.iter().find(|button| button.rect.contains(position))
            {
                return Some(button.spec.action);
            }
        }
        None
    }

    /// The space the menu adds beside the board.
    pub fn size(&self) -> Size {
        self.size
    }

    /// Whether focus is on the menu.
    pub fn has_focus(&self) -> bool {
        self.focus.is_some()
    }

    /// Returns focus to the board.
    pub fn clear_focus(&mut self) {
        self.focus = None;
    }

    /// Everywhere the focus can go, in the order it visits them: each
    /// tab's header, then its buttons while it's showing them.
    fn focus_order(&self) -> impl DoubleEndedIterator<Item = Focus> {
        self.tabs.iter().enumerate().flat_map(|(index, tab)| {
            let header = tab.header.is_some().then_some(Focus::Header(index));
            let buttons = tab.shown_buttons().is_some().then_some(Focus::Buttons(index));
            header.into_iter().chain(buttons)
        })
    }

    /// Moves focus on to the next place, returning it to the board once
    /// it runs off the last one.
    pub fn focus_next(&mut self) {
        self.focus = match self.focus {
            None => self.focus_order().next(),
            Some(current) => self.focus_order().skip_while(|focus| *focus != current).nth(1),
        };
    }

    /// The reverse of `focus_next`.
    pub fn focus_previous(&mut self) {
        self.focus = match self.focus {
            None => self.focus_order().next_back(),
            Some(current) => self.focus_order().rev().skip_while(|focus| *focus != current).nth(1),
        };
    }

    pub fn cursor_left(&mut self) {
        if let Some(Focus::Buttons(index)) = self.focus {
            self.tabs[index].cursor_previous();
        }
    }

    pub fn cursor_right(&mut self) {
        if let Some(Focus::Buttons(index)) = self.focus {
            self.tabs[index].cursor_next();
        }
    }

    /// Takes a press where the focus is. A header opens or closes its
    /// tab, as clicking it would, and returns no action. Buttons return
    /// the action of the one the cursor is on.
    pub fn press(&mut self) -> Option<Action<G>> {
        match self.focus? {
            Focus::Header(index) => {
                self.tabs[index].toggle_open();
                self.refresh_tabs();
                None
            }
            Focus::Buttons(index) => self.tabs[index].cursor_action(),
        }
    }
}

/// A tab's name and body. A named tab draws a header and can be
/// collapsed. An unnamed one is always just its body.
pub struct TabSpec<G> {
    pub name: Option<&'static str>,
    pub body: BodySpec<G>,
}

/// What a tab shows. Choices are buttons standing for one setting's
/// options, so each carries a tick column showing which is in use.
pub enum BodySpec<G> {
    Buttons(Vec<ButtonSpec<G>>),
    Choices(Vec<ButtonSpec<G>>),
    Legend(&'static [LegendGroup]),
}

/// A tab's header and body. An unnamed tab has no header. A closed one
/// has no body.
struct Tab<G> {
    spec: TabSpec<G>,
    open: bool,
    /// Where the cursor sits among the tab's buttons.
    cursor: usize,
    header: Option<Header>,
    body: Option<Body<G>>,
}

/// A tab's header: a line across the menu with its name written over
/// it.
struct Header {
    rect: Rect,
    name: &'static str,
}

/// A tab's body.
enum Body<G> {
    Buttons(Vec<Button<G>>),
    Legend(Legend),
}

impl<G: Copy> Tab<G> {
    fn toggle_open(&mut self) {
        self.open = !self.open;
    }

    fn shown_buttons(&self) -> Option<&[Button<G>]> {
        match &self.body {
            Some(Body::Buttons(buttons)) => Some(buttons),
            _ => None,
        }
    }

    fn cursor_action(&self) -> Option<Action<G>> {
        Some(self.shown_buttons()?[self.cursor].spec.action)
    }

    /// Moves the button cursor on by one, stopping at the last button.
    fn cursor_next(&mut self) {
        if let Some(buttons) = self.shown_buttons() {
            let last = buttons.len().saturating_sub(1);
            self.cursor = (self.cursor + 1).min(last);
        }
    }

    /// Moves the button cursor back by one, stopping at the first.
    fn cursor_previous(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }
}

impl<G: Copy> TabSpec<G> {
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

impl<G: Copy> BodySpec<G> {
    fn width(&self) -> usize {
        match self {
            BodySpec::Buttons(specs) => buttons_width(specs, false),
            BodySpec::Choices(specs) => buttons_width(specs, true),
            BodySpec::Legend(groups) => legend_width(groups),
        }
    }

    fn place(&self, top_left: Coord) -> Body<G> {
        match self {
            BodySpec::Buttons(specs) => Body::Buttons(place_buttons(specs, top_left, false)),
            BodySpec::Choices(specs) => Body::Buttons(place_buttons(specs, top_left, true)),
            BodySpec::Legend(groups) => Body::Legend(Legend { top_left, groups }),
        }
    }
}

impl<G> Body<G> {
    fn height(&self) -> usize {
        match self {
            Body::Buttons(_) => BUTTON_HEIGHT,
            Body::Legend(legend) => legend_height(legend.groups),
        }
    }
}

/// A legend's rows and where they start in the menu. Nothing in it is
/// clickable, so a corner to draw from is all it needs.
struct Legend {
    groups: &'static [LegendGroup],
    top_left: Coord,
}

pub struct LegendEntry {
    pub input: &'static str,
    pub description: &'static str,
}

/// A group of legend rows, under an optional label.
pub struct LegendGroup {
    pub label: Option<&'static str>,
    pub entries: &'static [LegendEntry],
}

/// What a button does. The `Game` variant carries an action only the
/// game itself knows, like a choice of how its board is drawn.
#[derive(Clone, Copy, Debug)]
pub enum Action<G> {
    NewGame,
    Restart,
    Undo,
    Redo,
    Solve,
    Quit,
    SetPreset(usize),
    Game(G),
}

/// Shown in a choice button's tick column when it's the current one.
const TICK: char = '✓';

/// What the menu shows of the game as it is right now: the preset the
/// game is at and whether there is a move to undo or redo.
#[derive(Clone, Copy, Debug, Default)]
pub struct MenuState {
    pub preset: Option<usize>,
    pub can_undo: bool,
    pub can_redo: bool,
}

impl MenuState {
    /// Reads the menu state off the session's current game.
    pub fn new<P>(session: &Session<P>) -> MenuState {
        MenuState {
            preset: session.which_preset(),
            can_undo: session.can_undo(),
            can_redo: session.can_redo(),
        }
    }
}

/// Whether a choice's action sets what the game already uses.
fn is_current<G: Copy>(
    action: Action<G>,
    menu_state: MenuState,
    is_game_action_current: impl Fn(G) -> bool,
) -> bool {
    match action {
        Action::SetPreset(id) => menu_state.preset == Some(id),
        Action::Game(action) => is_game_action_current(action),
        _ => false,
    }
}

#[derive(Clone, Debug)]
pub struct ButtonSpec<G> {
    action: Action<G>,
    label: String,
}

impl<G> ButtonSpec<G> {
    pub fn new(action: Action<G>, label: &str) -> ButtonSpec<G> {
        ButtonSpec { action, label: label.to_string() }
    }
}

/// One button of the menu, with the rectangle it occupies within it.
#[derive(Clone, Debug)]
struct Button<G> {
    spec: ButtonSpec<G>,
    rect: Rect,
    /// Whether the button is one of a set of choices.
    choice: bool,
}

/// The tabs of a game's menu, top to bottom. A game's own settings
/// tabs sit between the presets and the controls.
pub fn tabs<G>(
    presets: &[Preset],
    can_solve: bool,
    settings: Vec<TabSpec<G>>,
    game_controls: &'static [LegendGroup],
) -> Vec<TabSpec<G>> {
    let mut tabs = vec![
        TabSpec { name: None, body: BodySpec::Buttons(common_actions(can_solve)) },
        TabSpec { name: Some("Type"), body: BodySpec::Choices(preset_buttons(presets)) },
    ];

    tabs.extend(settings);
    tabs.push(TabSpec { name: Some("Menu Controls"), body: BodySpec::Legend(MENU_CONTROLS) });
    tabs.push(TabSpec { name: Some("Game Controls"), body: BodySpec::Legend(game_controls) });
    tabs
}

/// The row of actions at the top of a game's menu. Solve is left out
/// for a game with no solver.
fn common_actions<G>(can_solve: bool) -> Vec<ButtonSpec<G>> {
    let mut actions = vec![
        ButtonSpec::new(Action::NewGame, "New Game"),
        ButtonSpec::new(Action::Restart, "Restart"),
        ButtonSpec::new(Action::Undo, "Undo"),
        ButtonSpec::new(Action::Redo, "Redo"),
        ButtonSpec::new(Action::Quit, "Quit"),
    ];

    if can_solve {
        // Solve sits just before Quit.
        actions.insert(actions.len() - 1, ButtonSpec::new(Action::Solve, "Solve"));
    }
    actions
}

/// The presets the game offers, in the order the mid-end lists them.
fn preset_buttons<G>(presets: &[Preset]) -> Vec<ButtonSpec<G>> {
    presets
        .iter()
        .map(|preset| ButtonSpec::new(Action::SetPreset(preset.id), &preset.title))
        .collect()
}

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
    LegendEntry { input: "Enter / Space", description: "press a tab header or button" },
    LegendEntry { input: "Left mouse button", description: "" },
];

const MENU_CONTROLS: &[LegendGroup] = &[
    LegendGroup { label: Some("Game:"), entries: BOARD_SHORTCUTS },
    LegendGroup { label: Some("Menu:"), entries: MENU_KEYS },
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
        if entry.description.is_empty() {
            entry.input.chars().count()
        } else {
            input_width + LEGEND_SEPARATOR.chars().count() + entry.description.chars().count()
        }
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

/// Columns the menu starts with before its tabs: the divider itself,
/// with its margin either side.
const MENU_MARGIN: usize = 2 * DIVIDER_MARGIN + 1;

/// A box-drawn button's height: a border row above and below the row
/// holding the label.
const BUTTON_HEIGHT: usize = 3;

/// Every button pads a margin and a border column either side and a
/// choice adds a tick with a space before its label.
fn button_width<G>(spec: &ButtonSpec<G>, choice: bool) -> usize {
    let tick = if choice { 2 } else { 0 };
    spec.label.chars().count() + tick + 4
}

fn buttons_width<G>(specs: &[ButtonSpec<G>], choice: bool) -> usize {
    specs.iter().map(|spec| button_width(spec, choice)).sum()
}

fn place_buttons<G: Copy>(
    specs: &[ButtonSpec<G>],
    top_left: Coord,
    choice: bool,
) -> Vec<Button<G>> {
    let mut x = top_left.x;
    specs
        .iter()
        .map(|spec| {
            let size = Size::new(button_width(spec, choice), BUTTON_HEIGHT);
            let rect = Rect::new(Coord::new(x, top_left.y), size);
            x += size.width;
            Button { spec: spec.clone(), rect, choice }
        })
        .collect()
}

/// Draws a light vertical line down the menu's first columns, `length`
/// rows long.
fn draw_divider(canvas: &mut Canvas, offset: Coord, length: usize) {
    let column = offset.x + DIVIDER_MARGIN;
    let bottom = offset.y + length - 1;

    draw_line(canvas, (column, offset.y), (column, bottom), Weight::Light);
}

/// Draws a header: a line across the menu, the tab's name written
/// over it, and an arrow at the right end showing which way a click
/// will take it. The line and its arrow stop at the terminal's edge,
/// and the line is heavy while the tab has focus.
fn draw_header(
    canvas: &mut Canvas,
    header: &Header,
    offset: Coord,
    terminal_columns: usize,
    open: bool,
    focused: bool,
) {
    let rect = header.rect.shifted_by(offset);
    let row = rect.top();
    let right = rect.right().min(terminal_columns.saturating_sub(1));
    let weight = if focused {
        Weight::Heavy
    } else {
        Weight::Light
    };
    draw_line(canvas, (rect.left(), row), (right, row), weight);

    let name_start = Coord::new(rect.left() + HEADER_NAME_OFFSET, row);
    let name = format!(" {} ", header.name);
    canvas.draw_text_marked(name_start, &name, focused.then_some(Mark::Bold));

    canvas.draw_text((right, row), if open { "▲" } else { "▼" });
}

/// Draws a legend's groups from its top left, each label above its own
/// entries and a blank row between groups.
fn draw_legend(canvas: &mut Canvas, legend: &Legend, offset: Coord) {
    let input_width = widest_legend_input(legend.groups);
    let top_left = legend.top_left.shifted_by(offset);
    let start_column = top_left.x;
    let mut row = top_left.y;

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
            let text = if entry.description.is_empty() {
                entry.input.to_string()
            } else {
                let input = format!("{:<input_width$}", entry.input);
                format!("{input}{LEGEND_SEPARATOR}{}", entry.description)
            };
            canvas.draw_text((start_column, row), &text);
            row += 1;
        }
    }
}

/// Draws one button in its own rectangle, ticked if it is the choice
/// in use, dimmed if its action can't be taken and outlined heavy
/// while the cursor is on it.
fn draw_button<G: Copy>(
    canvas: &mut Canvas,
    button: &Button<G>,
    offset: Coord,
    has_cursor: bool,
    menu_state: MenuState,
    ticked: bool,
) {
    let rect = button.rect.shifted_by(offset);
    let weight = if has_cursor {
        Weight::Heavy
    } else {
        Weight::Light
    };
    draw_rect_outline(canvas, rect, weight);

    let label_start = Coord::new(rect.left() + 1, rect.top() + 1);
    let label = if button.choice {
        let tick = if ticked { TICK } else { ' ' };
        format!(" {tick} {} ", button.spec.label)
    } else {
        format!(" {} ", button.spec.label)
    };
    canvas.draw_text_marked(label_start, &label, has_cursor.then_some(Mark::Bold));

    if !available(button.spec.action, menu_state) {
        canvas.mark_region(rect.coords(), Mark::Dimmed);
    }
}

/// Whether an action can be taken right now. Every action but Undo and
/// Redo is always available.
fn available<G>(action: Action<G>, menu_state: MenuState) -> bool {
    match action {
        Action::Undo => menu_state.can_undo,
        Action::Redo => menu_state.can_redo,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Stands in for the actions a game defines.
    #[derive(Clone, Copy)]
    enum GameAction {
        First,
        Second,
    }

    /// Stands in for the inputs a game lists in its legend.
    const GAME_CONTROLS: &[LegendGroup] = &[LegendGroup {
        label: Some("Keyboard:"),
        entries: &[LegendEntry { input: "Arrows", description: "move cursor" }],
    }];

    /// Stands in for the settings tabs a game offers.
    fn settings() -> Vec<TabSpec<GameAction>> {
        let choices = vec![
            ButtonSpec::new(Action::Game(GameAction::First), "First"),
            ButtonSpec::new(Action::Game(GameAction::Second), "Second"),
        ];

        vec![TabSpec { name: Some("Choices"), body: BodySpec::Choices(choices) }]
    }

    /// Ten presets, the sizes a game might offer.
    fn presets() -> Vec<Preset> {
        let sizes = ["5x5", "7x7", "9x9", "11x11", "13x11"];
        let plain = sizes.iter().map(|size| size.to_string());
        let wrapping = sizes.iter().map(|size| format!("{size} wrapping"));

        plain
            .chain(wrapping)
            .enumerate()
            .map(|(id, title)| Preset { id, title })
            .collect()
    }

    /// A menu with a game's tabs, presets and all, every tab closed.
    fn menu() -> Menu<GameAction> {
        Menu::new(tabs(&presets(), true, settings(), GAME_CONTROLS))
    }

    /// The widest tab sets the width, open or not, so opening one
    /// never shifts the menu sideways. Type's ten presets take 143
    /// columns in a single row, far past every other tab.
    #[test]
    fn menu_width_is_its_widest_tab_and_the_margin() {
        assert_eq!(menu().size().width, 146);
    }

    /// A closed tab shows its header and nothing else.
    #[test]
    fn a_closed_tab_costs_only_its_header() {
        // The button row's three, and a header each for the four
        // named tabs.
        assert_eq!(menu().size().height, 7);
    }

    /// Clicking a header returns no action and opens the tab, making room
    /// for its body.
    #[test]
    fn clicking_a_header_opens_the_tab() {
        let mut menu = menu();
        // The Choices tab's header, two rows below the button row.
        assert!(menu.click((3, 4)).is_none());

        // Its buttons take one row, like any button body.
        assert_eq!(menu.size().height, 7 + BUTTON_HEIGHT);

        assert!(menu.click((3, 4)).is_none());
        assert_eq!(menu.size().height, 7);
    }

    /// A game with no solver gets every common action but Solve.
    #[test]
    fn solve_is_offered_only_with_a_solver() {
        let labels = |can_solve| {
            common_actions::<GameAction>(can_solve)
                .into_iter()
                .map(|spec| spec.label)
                .collect::<Vec<_>>()
        };

        assert_eq!(labels(true), ["New Game", "Restart", "Undo", "Redo", "Solve", "Quit"]);
        assert_eq!(labels(false), ["New Game", "Restart", "Undo", "Redo", "Quit"]);
    }

    fn unavailable_labels(menu_state: MenuState) -> Vec<String> {
        common_actions::<GameAction>(true)
            .into_iter()
            .filter(|spec| !available(spec.action, menu_state))
            .map(|spec| spec.label)
            .collect()
    }

    #[test]
    fn only_undo_and_redo_can_be_unavailable() {
        let neither = MenuState { can_undo: false, can_redo: false, ..MenuState::default() };
        assert_eq!(unavailable_labels(neither), ["Undo", "Redo"]);

        let both = MenuState { can_undo: true, can_redo: true, ..MenuState::default() };
        assert!(unavailable_labels(both).is_empty());
    }

    #[test]
    fn undo_and_redo_are_independent() {
        let undo_only = MenuState { can_undo: true, can_redo: false, ..MenuState::default() };
        assert_eq!(unavailable_labels(undo_only), ["Redo"]);
    }

    /// Where the buttons are placed is where clicks find them. The row
    /// starts after the divider's three columns, and the six buttons
    /// run to column 58.
    #[test]
    fn a_click_lands_on_the_button_it_is_over() {
        let mut menu = menu();
        let mut find = |x, y| menu.click((x, y));

        assert!(matches!(find(3, 0), Some(Action::NewGame)));
        assert!(matches!(find(14, 0), Some(Action::NewGame)));
        assert!(matches!(find(15, 0), Some(Action::Restart)));
        assert!(matches!(find(58, 2), Some(Action::Quit)));

        // Left of the row, past its end, and below every tab.
        assert!(find(2, 0).is_none());
        assert!(find(59, 0).is_none());
        assert!(find(3, 7).is_none());
    }

    /// Clicks arrive in screen coordinates, so a moved menu takes them
    /// where it now sits and ignores the spot it left.
    #[test]
    fn clicks_find_a_moved_menu_where_it_sits() {
        let mut menu = menu();
        menu.set_placement(Rect::new(Coord::new(0, 0), Size::new(25, 14)), usize::MAX);

        assert!(matches!(menu.click((28, 0)), Some(Action::NewGame)));
        assert!(menu.click((3, 0)).is_none());
    }

    /// The 'Tab' key visits each header and each row of buttons on
    /// show, in the order they are drawn, then hands the focus back to
    /// the board.
    #[test]
    fn focus_visits_every_header_and_row_of_buttons() {
        let mut menu = menu();
        let mut visited = Vec::new();
        for _ in 0..6 {
            menu.focus_next();
            visited.push(menu.focus);
        }

        assert_eq!(
            visited,
            [
                Some(Focus::Buttons(0)),
                Some(Focus::Header(1)),
                Some(Focus::Header(2)),
                Some(Focus::Header(3)),
                Some(Focus::Header(4)),
                None,
            ]
        );
    }

    /// A legend has nothing to press, so opening one gives the focus
    /// nowhere new to stop, where opening a row of buttons does.
    #[test]
    fn only_buttons_give_the_focus_somewhere_to_stop() {
        let mut menu = menu();
        let closed = menu.focus_order().count();

        // Menu Controls, the third header down.
        menu.click((3, 5));
        assert_eq!(menu.focus_order().count(), closed);

        // Choices, the second.
        menu.click((3, 4));
        assert_eq!(menu.focus_order().count(), closed + 1);
    }

    /// 'Shift + Tab' reverses 'Tab' and from the board it lands on the
    /// last place.
    #[test]
    fn focus_previous_reverses_focus_next() {
        let mut menu = menu();
        menu.focus_next();
        menu.focus_next();
        let two_in = menu.focus;

        menu.focus_next();
        menu.focus_previous();
        assert_eq!(menu.focus, two_in);

        menu.clear_focus();
        menu.focus_previous();
        assert_eq!(menu.focus, Some(Focus::Header(4)));
    }

    /// A press on a header opens or closes its tab and gives no
    /// action, where a press on buttons gives the one under the cursor.
    #[test]
    fn pressing_a_header_opens_it_and_pressing_buttons_acts() {
        let mut menu = menu();
        let closed_height = menu.size().height;

        // The Choices tab's header, the third place the focus visits.
        menu.focus_next();
        menu.focus_next();
        menu.focus_next();
        assert!(menu.press().is_none());
        assert_eq!(menu.size().height, closed_height + BUTTON_HEIGHT);

        assert!(menu.press().is_none());
        assert_eq!(menu.size().height, closed_height);

        menu.clear_focus();
        menu.focus_next();
        assert!(matches!(menu.press(), Some(Action::NewGame)));
    }

    /// A preset's button is ticked only while the game is at it.
    #[test]
    fn only_the_current_preset_is_ticked() {
        let at_third = MenuState { preset: Some(3), ..MenuState::default() };
        let custom = MenuState { preset: None, ..MenuState::default() };

        let ticked = |action, menu_state| is_current::<GameAction>(action, menu_state, |_| false);

        assert!(ticked(Action::SetPreset(3), at_third));
        assert!(!ticked(Action::SetPreset(4), at_third));
        assert!(!ticked(Action::SetPreset(3), custom));
    }
}

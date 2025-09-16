use iced::widget::{
    button, center_y, column, container, responsive, row, scrollable, text,
    pane_grid::{self, PaneGrid},
};
use iced::{keyboard, window, Color, Element, Fill, Size, Subscription, Task, Theme};
use std::collections::HashMap;

#[derive(Clone, Copy)]
struct Pane {
    pub id: usize,
    pub is_pinned: bool,
    //pub pane_type: PaneEnum,
}
impl Pane {
    fn new(id: usize) -> Self { 
        Self { 
            id, 
            is_pinned: false 
        } 
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PaneMsg {
    // PaneGrid actions
    Split(pane_grid::Axis, pane_grid::Pane),
    SplitFocused(pane_grid::Axis),
    FocusAdjacent(pane_grid::Direction),
    Clicked(pane_grid::Pane),
    Dragged(pane_grid::DragEvent),
    Resized(pane_grid::ResizeEvent),
    TogglePin(pane_grid::Pane),
    Maximize(pane_grid::Pane),
    Restore,
    Close(pane_grid::Pane),
    CloseFocused,

    // Detach / attach
    PopOut(pane_grid::Pane),
    DetachedOpened { win: window::Id, pane: pane_grid::Pane },
    DockBack(window::Id),

    // Window lifecycle
    RegisteredMain(window::Id),
    WindowClosed(window::Id),
}

pub struct PaneDock {
    main: Option<window::Id>,
    panes: pane_grid::State<Pane>,
    panes_created: usize,
    focus: Option<pane_grid::Pane>,
    detached: HashMap<window::Id, Pane>,
}

impl PaneDock {
    /// Host-managed: you already have a main window; set it here.
    pub fn new_with_main(main: window::Id) -> Self {
        let (panes, _) = pane_grid::State::new(Pane::new(0));
        Self {
            main: Some(main),
            panes,
            panes_created: 1,
            focus: None,
            detached: Default::default(),
        }
    }

    /// Self-managed: opens its own main window and returns the Task that yields its Id.
    pub fn new_open_main() -> (Self, Task<PaneMsg>) {
        let (panes, _) = pane_grid::State::new(Pane::new(0));
        let mut s = Self {
            main: None,
            panes,
            panes_created: 1,
            focus: None,
            detached: Default::default(),
        };
        let (_id, open) = window::open(window::Settings {
            size: Size::new(960.0, 640.0),
            ..Default::default()
        });
        (s, open.then(|id| Task::done(PaneMsg::RegisteredMain(id))))
    }

    /// Let the host know if *this* module should draw a given window.
    pub fn owns_window(&self, id: window::Id) -> bool {
        self.main == Some(id) || self.detached.contains_key(&id)
    }

    pub fn update(&mut self, message: PaneMsg) -> Task<PaneMsg> {
        use pane_grid::{Axis, Direction};
        match message {
            PaneMsg::RegisteredMain(id) => { self.main = Some(id); Task::none() }
            PaneMsg::WindowClosed(id) => { self.detached.remove(&id); Task::none() }

            PaneMsg::PopOut(pane) => {
                let Some(pane_data) = self.panes.get(pane).copied() else { return Task::none(); };
                let (_id, open) = window::open(window::Settings {
                    size: Size::new(480.0, 320.0),
                    ..Default::default()
                });
                open.then(move |win| Task::done(PaneMsg::DetachedOpened { win, pane }))
            }
            PaneMsg::DetachedOpened { win, pane } => {
                if let Some(p) = self.panes.get(pane).copied() {
                    self.detached.insert(win, p);
                }
                if let Some((_, sib)) = self.panes.close(pane) {
                    self.focus = Some(sib);
                }
                Task::none()
            }
            PaneMsg::DockBack(win) => {
                if let Some(pane_data) = self.detached.remove(&win) {
                    if let Some(target) = self.focus {
                        let _ = self.panes.split(pane_grid::Axis::Vertical, target, pane_data);
                    }
                }
                window::close(win)
            }

            PaneMsg::Split(axis, pane) => {
                if let Some((p, _)) = self.panes.split(axis, pane, Pane::new(self.panes_created)) {
                    self.focus = Some(p);
                }
                self.panes_created += 1;
                Task::none()
            }
            PaneMsg::SplitFocused(axis) => {
                if let Some(p) = self.focus {
                    if let Some((p2, _)) = self.panes.split(axis, p, Pane::new(self.panes_created)) {
                        self.focus = Some(p2);
                    }
                    self.panes_created += 1;
                }
                Task::none()
            }
            PaneMsg::FocusAdjacent(dir) => {
                if let Some(p) = self.focus
                    && let Some(adj) = self.panes.adjacent(p, dir)
                {
                    self.focus = Some(adj);
                }
                Task::none()
            }
            PaneMsg::Clicked(p) => { self.focus = Some(p); Task::none() }
            PaneMsg::Resized(pane_grid::ResizeEvent { split, ratio }) => { self.panes.resize(split, ratio); Task::none() }
            PaneMsg::Dragged(pane_grid::DragEvent::Dropped { pane, target }) => { self.panes.drop(pane, target); Task::none() }
            PaneMsg::Dragged(_) => Task::none(),
            PaneMsg::TogglePin(p) => {
                if let Some(Pane { is_pinned, .. }) = self.panes.get_mut(p) {
                    *is_pinned = !*is_pinned;
                }
                Task::none()
            }
            PaneMsg::Maximize(p) => { self.panes.maximize(p); Task::none() }
            PaneMsg::Restore => { self.panes.restore(); Task::none() }
            PaneMsg::Close(p) => {
                if let Some((_, sib)) = self.panes.close(p) {
                    self.focus = Some(sib);
                }
                Task::none()
            }
            PaneMsg::CloseFocused => {
                if let Some(p) = self.focus
                    && let Some(Pane { is_pinned, .. }) = self.panes.get(p)
                    && !is_pinned
                    && let Some((_, sib)) = self.panes.close(p)
                {
                    self.focus = Some(sib);
                }
                Task::none()
            }
        }
    }

    pub fn subscription(&self) -> Subscription<PaneMsg> {
        Subscription::batch(vec![
            keyboard::on_key_press(|key_code, modifiers| {
                use iced::keyboard::key::{self, Key};
                use pane_grid::{Axis, Direction};
                if !modifiers.command() { return None; }
                match key_code.as_ref() {
                    Key::Character("v") => Some(PaneMsg::SplitFocused(Axis::Vertical)),
                    Key::Character("h") => Some(PaneMsg::SplitFocused(Axis::Horizontal)),
                    Key::Character("w") => Some(PaneMsg::CloseFocused),
                    Key::Named(k) => {
                        let dir = match k {
                            key::Named::ArrowUp => Some(Direction::Up),
                            key::Named::ArrowDown => Some(Direction::Down),
                            key::Named::ArrowLeft => Some(Direction::Left),
                            key::Named::ArrowRight => Some(Direction::Right),
                            _ => None,
                        };
                        dir.map(PaneMsg::FocusAdjacent)
                    }
                    _ => None,
                }
            }),
            window::close_events().map(PaneMsg::WindowClosed),
        ])
    }

    /// Draw a window this module owns.
    pub fn view(&self, which: window::Id) -> Element<'_, PaneMsg> {
        match self.main {
            Some(main_id) if which == main_id => self.view_main(),
            _ => self.view_detached(which),
        }
    }

    // --- internals ---

    fn view_main(&self) -> Element<'_, PaneMsg> {
        use pane_grid as pg;
        let focus = self.focus;
        let total_panes = self.panes.len();

        let grid = PaneGrid::new(&self.panes, move |id, pane, is_maximized| {
            let is_focused = focus == Some(id);

            let pin_button = button(text(if pane.is_pinned { "Unpin" } else { "Pin" }).size(14))
                .on_press(PaneMsg::TogglePin(id))
                .padding(3);

            let pop_button = button(text("Pop out").size(14))
                .padding(3)
                .on_press_maybe(if !pane.is_pinned { Some(PaneMsg::PopOut(id)) } else { None });

            let title = row![
                pin_button,
                pop_button,
                "Pane",
                text(pane.id.to_string()).color(if is_focused { PANE_ID_COLOR_FOCUSED } else { PANE_ID_COLOR_UNFOCUSED }),
            ]
            .spacing(5);

            let title_bar = pg::TitleBar::new(title)
                .padding(10)
                .style(if is_focused { style::title_bar_focused } else { style::title_bar_active });

            pg::Content::new(responsive(move |size| view_content(id, total_panes, pane.is_pinned, size)))
                .title_bar(title_bar)
                .style(if is_focused { style::pane_focused } else { style::pane_active })
        })
        .width(Fill)
        .height(Fill)
        .spacing(10)
        .on_click(PaneMsg::Clicked)
        .on_drag(PaneMsg::Dragged)
        .on_resize(10, PaneMsg::Resized);

        container(grid).padding(10).into()
    }

    fn view_detached(&self, win: window::Id) -> Element<'_, PaneMsg> {
        if let Some(pane) = self.detached.get(&win) {
            let body = column![
                text(format!("Detached pane {}", pane.id)).size(18),
                center_y(scrollable(column![ view_controls_detached(win), ].spacing(8).max_width(220)))
            ]
            .spacing(10);
            container(body).padding(10).into()
        } else {
            container(text("Window closed or unknown")).padding(10).into()
        }
    }
}

fn view_controls_detached(win: window::Id) -> Element<'static, PaneMsg> {
    let dock = button(text("Dock back").size(14))
        .padding(6)
        .on_press(PaneMsg::DockBack(win));
    row![dock].spacing(6).into()
}

fn view_content<'a>(
    pane: pane_grid::Pane,
    total_panes: usize,
    is_pinned: bool,
    size: Size,
) -> Element<'a, PaneMsg> {
    let b = |label, msg| {
        button(text(label).width(Fill).align_x(iced::Center).size(16))
            .width(Fill)
            .padding(8)
            .on_press(msg)
    };

    let controls = column![
        b("Split horizontally", PaneMsg::Split(pane_grid::Axis::Horizontal, pane)),
        b("Split vertically",   PaneMsg::Split(pane_grid::Axis::Vertical, pane)),
        if total_panes > 1 && !is_pinned {
            Some(b("Close", PaneMsg::Close(pane)).style(button::danger))
        } else { None }
    ]
    .spacing(5)
    .max_width(160);

    let content = column![text!("{}x{}", size.width, size.height).size(24), controls]
        .spacing(10)
        .align_x(iced::Center);

    center_y(scrollable(content)).padding(5).into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneEnum {
    Visualizer,     // UI Preview
    Editor,         // Widget Property Editor
    Tree,           // Tree for adding widgets to the preview / updating the Editor
    Code,           // Full app code
    Training,       // Future idea to walk people through the basics of iced-rs
}

mod style {
    use iced::widget::container;
    use iced::{Border, Theme};
    pub fn title_bar_active(theme: &Theme) -> container::Style {
        let p = theme.extended_palette();
        container::Style {
            text_color: Some(p.background.strong.text),
            background: Some(p.background.strong.color.into()),
            ..Default::default()
        }
    }
    pub fn title_bar_focused(theme: &Theme) -> container::Style {
        let p = theme.extended_palette();
        container::Style {
            text_color: Some(p.primary.strong.text),
            background: Some(p.primary.strong.color.into()),
            ..Default::default()
        }
    }
    pub fn pane_active(theme: &Theme) -> container::Style {
        let p = theme.extended_palette();
        container::Style {
            background: Some(p.background.weak.color.into()),
            border: Border { width: 2.0, color: p.background.strong.color, ..Border::default() },
            ..Default::default()
        }
    }
    pub fn pane_focused(theme: &Theme) -> container::Style {
        let p = theme.extended_palette();
        container::Style {
            background: Some(p.background.weak.color.into()),
            border: Border { width: 2.0, color: p.primary.strong.color, ..Border::default() },
            ..Default::default()
        }
    }
}

// public so host can reuse if desired
pub const PANE_ID_COLOR_UNFOCUSED: Color = Color::from_rgb(1.0, 0xC7 as f32 / 255.0, 0xC7 as f32 / 255.0);
pub const PANE_ID_COLOR_FOCUSED:   Color = Color::from_rgb(1.0, 0x47 as f32 / 255.0, 0x47 as f32 / 255.0);
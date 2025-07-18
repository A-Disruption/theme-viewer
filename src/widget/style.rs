use iced::{Background, Border, Color, Theme};
use crate::widget::status::Status;

/// The appearance of a [`ColorPicker`].
#[derive(Debug, Clone, Copy)]
pub struct Style {
    /// The background of the [`ColorPicker`]
    pub background: Background,

    /// The border radius of the [`ColorPicker`]
    pub border_radius: f32,

    /// The border with of the [`ColorPicker`]
    pub border_width: f32,

    /// The border color of the [`ColorPicker`]
    pub border_color: Color,

    /// The border radius of the bars of the [`ColorPicker`]
    pub bar_border_radius: f32,

    /// The border width of the bars of the [`ColorPicker`]
    pub bar_border_width: f32,

    /// The border color of the bars of the [`ColorPicker`]
    pub bar_border_color: Color,
}

/// The state of the style
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StyleState {
    /// Use the active style
    Active,
    /// Use the selected style
    Selected,
    /// Use the hovered style
    Hovered,
    /// Use the focused style
    Focused,
}

/// The theme catalog of a [`ColorPicker`].
pub trait Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> <Self as Catalog>::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &<Self as Catalog>::Class<'_>, status: Status) -> Style;
}

/// A styling function for a [`ColorPicker`].
pub type StyleFn<'a, Theme, Status> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self, Status>;

    fn default<'a>() -> StyleFn<'a, Self, Status> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

/// The default style of a [`ColorPicker`].
pub fn default(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let foreground = theme.palette();

    let base = Style {
        background: palette.background.strong.color.into(),
        border_radius: 15.0,
        border_width: 1.0,
        border_color: foreground.text,
        bar_border_radius: 5.0,
        bar_border_width: 1.0,
        bar_border_color: foreground.text,
    };

    match status {
        Status::Focused => Style {
            border_color: palette.background.strong.color,
            bar_border_color: palette.background.strong.color,
            ..base
        },
        _ => base,
    }
}
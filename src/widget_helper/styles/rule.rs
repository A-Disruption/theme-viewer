pub use iced::widget::rule::*;
use iced::{border::{radius, Radius}, Border, Theme};

/// A [`Rule`] styling using the weak background color.
pub fn toolbar_rule(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    Style {
        color: palette.background.weak.color,
        radius: 0.0.into(),
        fill_mode: FillMode::Percent(80.0),
        snap: true,
    }
}
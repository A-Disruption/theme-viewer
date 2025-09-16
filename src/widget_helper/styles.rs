pub use iced::widget::button::*;
use iced::{border::{radius, Radius}, Border, Theme};


pub fn cancel(theme: &Theme, status: Status) -> Style {

    let palette = theme.extended_palette();

    let base = Style {
        text_color: palette.danger.strong.color,
/*         border: Border {
            color: palette.secondary.base.color,
            width: 1.0,
            radius: radius(4.0)
        }, */
        ..Style::default()
    };

    match status {
        Status::Active | Status::Pressed => base,
        Status::Hovered => Style {
            text_color: palette.danger.strong.color.scale_alpha(0.8),
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

pub fn save(theme: &Theme, status: Status) -> Style {

    let palette = theme.extended_palette();

    let base = Style {
        text_color: palette.primary.strong.color,
        border: Border {
            color: palette.secondary.base.color,
            width: 1.0,
            radius: radius(4.0)
        },
        ..Style::default()
    };

    match status {
        Status::Active | Status::Pressed => base,
        Status::Hovered => Style {
            text_color: palette.primary.strong.color.scale_alpha(0.8),
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

fn disabled(style: Style) -> Style {
    Style {
        background: style
            .background
            .map(|background| background.scale_alpha(0.5)),
        text_color: style.text_color.scale_alpha(0.5),
        ..style
    }
}
// Generated automatically by iced_fontello at build time.
// Do not edit manually. Source: ../fonts/fonts.toml
// a802b031cbe45d3e3659b50cff9e48d674685cb97d28b158f38351bc28a6fb0b
use iced::widget::{text, Text};
use iced::Font;

pub const FONT: &[u8] = include_bytes!("../fonts/fonts.ttf");

pub fn code<'a>() -> Text<'a> {
    icon("\u{F1C9}")
}

pub fn cog<'a>() -> Text<'a> {
    icon("\u{2699}")
}

pub fn copy<'a>() -> Text<'a> {
    icon("\u{F0C5}")
}

pub fn edit<'a>() -> Text<'a> {
    icon("\u{270E}")
}

pub fn global<'a>() -> Text<'a> {
    icon("\u{1F30E}")
}

pub fn home<'a>() -> Text<'a> {
    icon("\u{2302}")
}

pub fn plus<'a>() -> Text<'a> {
    icon("\u{2B}")
}

pub fn preview<'a>() -> Text<'a> {
    icon("\u{1F304}")
}

pub fn swap<'a>() -> Text<'a> {
    icon("\u{F0EC}")
}

pub fn theme<'a>() -> Text<'a> {
    icon("\u{E032}")
}

pub fn trash<'a>() -> Text<'a> {
    icon("\u{F1F8}")
}

fn icon(codepoint: &str) -> Text<'_> {
    text(codepoint).font(Font::with_name("fonts"))
}

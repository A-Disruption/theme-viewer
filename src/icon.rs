// Generated automatically by iced_fontello at build time.
// Do not edit manually. Source: ../fonts/fonts.toml
// a94c08cbdc34881f47f15d9df59536fdbc41a9172d08899275abb0b5b609137c
use iced::widget::{text, Text};
use iced::Font;

pub const FONT: &[u8] = include_bytes!("../fonts/fonts.ttf");

pub fn code<'a>() -> Text<'a> {
    icon("\u{F1C9}")
}

pub fn cog<'a>() -> Text<'a> {
    icon("\u{2699}")
}

pub fn collapsed<'a>() -> Text<'a> {
    icon("\u{25B8}")
}

pub fn copy<'a>() -> Text<'a> {
    icon("\u{F0C5}")
}

pub fn edit<'a>() -> Text<'a> {
    icon("\u{270E}")
}

pub fn expanded<'a>() -> Text<'a> {
    icon("\u{25BE}")
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

pub fn save<'a>() -> Text<'a> {
    icon("\u{1F4BE}")
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

pub fn type_icon<'a>() -> Text<'a> {
    icon("\u{F0F7}")
}

fn icon(codepoint: &str) -> Text<'_> {
    text(codepoint).font(Font::with_name("fonts"))
}

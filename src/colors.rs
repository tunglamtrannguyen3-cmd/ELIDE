use crossterm::style::{Attribute, Color};

pub struct Palette;
impl Palette {
    pub const ERROR_RED: Color = Color::Rgb { r: 227, g: 83, b: 54 };
    pub const WARNING_GOLD: Color = Color::Rgb { r: 255, g: 215, b: 0 };
    pub const HINT_ICE_BLUE: Color = Color::Rgb { r: 193, g: 213, b: 240 };
    pub const SUCCESS_LIME: Color = Color::Rgb { r: 137, g: 243, b: 54 };
    pub const NAVY_GRAY: Color = Color::Rgb { r: 112, g: 128, b: 144 };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Success,
    Warning,
    Error,
    UnknownCommand,
    UnknownCode,
}

pub fn color_for_status(status: Status) -> Color {
    match status {
        Status::Success => Palette::SUCCESS_LIME,
        Status::Warning => Palette::WARNING_GOLD,
        Status::Error => Palette::ERROR_RED,
        Status::UnknownCommand | Status::UnknownCode => Palette::NAVY_GRAY,
    }
}

pub fn attribute_for_status(status: Status) -> Attribute {
    match status {
        Status::Success | Status::Warning | Status::Error => Attribute::Bold,
        Status::UnknownCommand | Status::UnknownCode => Attribute::Dim,
    }
}

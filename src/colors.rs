use crossterm::style::{Attribute, Color};

pub struct Palette;

impl Palette {
    // --- Status Colors ---
    pub const ERROR_RED: Color = Color::Rgb { r: 227, g: 83, b: 54 };
    pub const WARNING_GOLD: Color = Color::Rgb { r: 255, g: 215, b: 0 };
    pub const HINT_ICE_BLUE: Color = Color::Rgb { r: 193, g: 213, b: 240 };
    pub const SUCCESS_LIME: Color = Color::Rgb { r: 137, g: 243, b: 54 };
    pub const NAVY_GRAY: Color = Color::Rgb { r: 112, g: 128, b: 144 };

    // --- Relaxing Syntax Highlighting Colors ---
    pub const TEXT_DEFAULT: Color = Color::Rgb { r: 200, g: 204, b: 214 }; // Soft neutral gray
    pub const KEYWORD_PURPLE: Color = Color::Rgb { r: 184, g: 176, b: 210 }; // Dusty lavender
    pub const STRING_GREEN: Color = Color::Rgb { r: 164, g: 190, b: 166 }; // Muted sage
    pub const COMMENT_GRAY: Color = Color::Rgb { r: 125, g: 130, b: 145 }; // Quiet slate
    pub const FUNCTION_BLUE: Color = Color::Rgb { r: 157, g: 180, b: 202 }; // Desaturated blue-gray
    pub const TYPE_YELLOW: Color = Color::Rgb { r: 205, g: 195, b: 158 }; // Muted warm sand
    pub const NUMBER_ORANGE: Color = Color::Rgb { r: 205, g: 166, b: 143 }; // Dusty peach
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Success,
    Warning,
    Error,
    Hint,
    UnknownCommand,
    UnknownCode,
}

pub fn color_for_status(status: Status) -> Color {
    match status {
        Status::Success => Palette::SUCCESS_LIME,
        Status::Warning => Palette::WARNING_GOLD,
        Status::Error => Palette::ERROR_RED,
        Status::Hint => Palette::HINT_ICE_BLUE,
        Status::UnknownCommand | Status::UnknownCode => Palette::NAVY_GRAY,
    }
}

pub fn attribute_for_status(status: Status) -> Attribute {
    match status {
        Status::Success | Status::Warning | Status::Error | Status::Hint => Attribute::Bold,
        Status::UnknownCommand | Status::UnknownCode => Attribute::Dim,
    }
}
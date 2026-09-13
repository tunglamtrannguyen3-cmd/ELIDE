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
    pub const TEXT_DEFAULT: Color = Color::Rgb { r: 205, g: 214, b: 244 }; // Gentle off-white
    pub const KEYWORD_PURPLE: Color = Color::Rgb { r: 203, g: 166, b: 247 }; // Soft lavender (`fn`, `let`, `pub`)
    pub const STRING_GREEN: Color = Color::Rgb { r: 166, g: 227, b: 161 };  // Pastel mint (`"hello"`)
    pub const COMMENT_GRAY: Color = Color::Rgb { r: 147, g: 153, b: 178 };  // Muted slate gray (`// ...`)
    pub const FUNCTION_BLUE: Color = Color::Rgb { r: 137, g: 180, b: 250 }; // Calm sky blue (`main()`)
    pub const TYPE_YELLOW: Color = Color::Rgb { r: 249, g: 226, b: 175 };    // Warm cream (`String`, `usize`)
    pub const NUMBER_ORANGE: Color = Color::Rgb { r: 250, g: 179, b: 135 };  // Soft peach (`42`, `0xFF`)
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
use crossterm::style::{Attribute, Color};

pub struct Palette;

impl Palette {
    // Relaxing Catppuccin-style pastels
    pub const KEYWORD_MAUVE: Color = Color::Rgb { r: 203, g: 166, b: 247 };
    pub const TYPE_YELLOW: Color = Color::Rgb { r: 249, g: 226, b: 175 };
    pub const NUMBER_PEACH: Color = Color::Rgb { r: 250, g: 179, b: 135 };
    pub const STRING_GREEN: Color = Color::Rgb { r: 166, g: 227, b: 161 };
    pub const FUNCTION_BLUE: Color = Color::Rgb { r: 137, g: 180, b: 250 };
    pub const TEXT_DEFAULT: Color = Color::Rgb { r: 205, g: 214, b: 244 };
    pub const COMMENT_GRAY: Color = Color::Rgb { r: 147, g: 153, b: 178 };
    pub const UI_BORDER: Color = Color::Rgb { r: 88, g: 91, b: 112 };
    
    // Status colors for LSP and Terminal
    pub const SUCCESS_GREEN: Color = Color::Rgb { r: 166, g: 227, b: 161 };
    pub const ERROR_RED: Color = Color::Rgb { r: 243, g: 139, b: 168 };
    pub const WARNING_YELLOW: Color = Color::Rgb { r: 250, g: 227, b: 176 };
    pub const HINT_TEAL: Color = Color::Rgb { r: 148, g: 226, b: 213 };
    pub const UNKNOWN_FLAMINGO: Color = Color::Rgb { r: 242, g: 205, b: 205 };
}

#[derive(Clone, Copy)]
pub enum Status {
    Success,
    Error,
    Warning,
    Hint,
    UnknownCommand,
    UnknownCode,
}

pub fn color_for_status(status: Status) -> Color {
    match status {
        Status::Success => Palette::SUCCESS_GREEN,
        Status::Error => Palette::ERROR_RED,
        Status::Warning => Palette::WARNING_YELLOW,
        Status::Hint => Palette::HINT_TEAL,
        Status::UnknownCommand | Status::UnknownCode => Palette::UNKNOWN_FLAMINGO,
    }
}

pub fn attribute_for_status(status: Status) -> Attribute {
    match status {
        Status::Error | Status::UnknownCommand | Status::Warning => Attribute::Bold,
        _ => Attribute::Reset,
    }
}
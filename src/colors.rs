// src/colors.rs
use crossterm::style::{Attribute, Color};

/// ELIDE Serenity — muted lavender, misty blue, and warm pastels.
pub struct Palette;

impl Palette {
    // Editor Backgrounds
    pub const BG_MAIN: Color = Color::Rgb { r: 30, g: 30, b: 46 };     // #1E1E2E Catppuccin Base
    pub const BG_PANEL: Color = Color::Rgb { r: 24, g: 24, b: 37 };    // #181825 Side/LSP/Terminal Pane

    // Syntax Highlighting
    pub const KEYWORD_MAUVE: Color = Color::Rgb { r: 198, g: 160, b: 246 }; // #C6A0F6 Soft Lavender
    pub const TYPE_YELLOW: Color = Color::Rgb { r: 232, g: 207, b: 161 };   // #E8CFA1 Warm Sand
    pub const NUMBER_PEACH: Color = Color::Rgb { r: 233, g: 173, b: 145 };  // #E9AD91 Muted Apricot
    pub const STRING_GREEN: Color = Color::Rgb { r: 166, g: 214, b: 174 };  // #A6D6AE Sage Green
    pub const FUNCTION_BLUE: Color = Color::Rgb { r: 154, g: 188, b: 235 }; // #9ABCEB Misty Blue
    pub const TEXT_DEFAULT: Color = Color::Rgb { r: 197, g: 206, b: 229 };  // #C5CEE5 Moonlight
    pub const COMMENT_GRAY: Color = Color::Rgb { r: 148, g: 155, b: 181 };  // #949BB5 Dusty Blue-Gray
    pub const UI_BORDER: Color = Color::Rgb { r: 69, g: 72, b: 94 };        // #45485E Quiet Slate

    // ELIDE Signature Accent
    pub const SOFT_NAVY: Color = Color::Rgb { r: 163, g: 185, b: 217 };     // #A3B9D9 Function Signatures

    // Status Colors
    pub const SUCCESS_GREEN: Color = Color::Rgb { r: 166, g: 214, b: 174 }; // #A6D6AE Sage Green
    pub const ERROR_RED: Color = Color::Rgb { r: 230, g: 160, b: 181 };     // #E6A0B5 Soft Rose Error
    pub const WARNING_YELLOW: Color = Color::Rgb { r: 232, g: 207, b: 161 };// #E8CFA1 Warm Sand Warning
    pub const HINT_TEAL: Color = Color::Rgb { r: 146, g: 212, b: 208 };     // #92D4D0 Soft Teal
    pub const UNKNOWN_FLAMINGO: Color = Color::Rgb { r: 229, g: 182, b: 194 }; // #E5B6C2 Muted Flamingo
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
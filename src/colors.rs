use ratatui::style::{Color, Modifier, Style};

pub struct Palette;

impl Palette {
    pub const ERROR_RED: Color = Color::Rgb(255, 107, 107);    // #FF6B6B
    pub const WARNING_AMBER: Color = Color::Rgb(255, 184, 77);  // #FFB84D
    pub const SUCCESS_MINT: Color = Color::Rgb(78, 206, 144);   // #4ECE90
    pub const NAVY_GRAY: Color = Color::Rgb(112, 128, 144);    // #708090 (Navy / Slate Gray)
}

pub fn error_style() -> Style {
    Style::default()
        .fg(Palette::ERROR_RED)
        .add_modifier(Modifier::BOLD)
}

pub fn warning_style() -> Style {
    Style::default()
        .fg(Palette::WARNING_AMBER)
        .add_modifier(Modifier::BOLD)
}

pub fn success_style() -> Style {
    Style::default().fg(Palette::SUCCESS_MINT)
}

pub fn navy_gray_style() -> Style {
    Style::default()
        .fg(Palette::NAVY_GRAY)
        .add_modifier(Modifier::DIM)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Success,
    Warning,
    Error,
    UnknownCommand,
    UnknownCode,
}

pub fn style_for_status(status: Status) -> Style {
    match status {
        Status::Success => success_style(),
        Status::Warning => warning_style(),
        Status::Error => error_style(),
        Status::UnknownCommand | Status::UnknownCode => navy_gray_style(),
    }
}


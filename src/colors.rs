use ratatui::style::{Color, Modifier, Style};

pub struct Palette;

impl Palette {
    pub const ERROR_RED: Color = Color::Rgb(227, 83, 54);      // #E35336
    pub const WARNING_GOLD: Color = Color::Rgb(255, 215, 0);    // #FFD700
    pub const HINT_ICE_BLUE: Color = Color::Rgb(193, 213, 240);  // #C1D5F0
    pub const SUCCESS_LIME: Color = Color::Rgb(137, 243, 54);   // #89F336
    pub const NAVY_GRAY: Color = Color::Rgb(112, 128, 144);     // #708090
}

pub fn error_style() -> Style {
    Style::default()
        .fg(Palette::ERROR_RED)
        .add_modifier(Modifier::BOLD)
}

pub fn warning_style() -> Style {
    Style::default()
        .fg(Palette::WARNING_GOLD)
        .add_modifier(Modifier::BOLD)
}

pub fn hint_style() -> Style {
    Style::default().fg(Palette::HINT_ICE_BLUE)
}

pub fn success_style() -> Style {
    Style::default()
        .fg(Palette::SUCCESS_LIME)
        .add_modifier(Modifier::BOLD)
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

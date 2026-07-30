use ratatui::style::{Color, Style, Modifier};

/// ELIDE Custom Palette - High Contrast, Minimal Eye-Strain RGB Constants
pub struct Palette;

impl Palette {
    // Màu chuẩn đơn sắc - Tối ưu cho text không nền (Zero-Background)
    pub const ERROR_RED: Color = Color::Rgb(255, 107, 107);    // #FF6B6B
    pub const WARNING_AMBER: Color = Color::Rgb(255, 184, 77);  // #FFB84D
    pub const SUCCESS_MINT: Color = Color::Rgb(78, 206, 144);  // #4ECE90
}

// Errors: Bold đỏ san hô nổi bật
pub fn error_style() -> Style {
    Style::default()
        .fg(Palette::ERROR_RED)
        .add_modifier(Modifier::BOLD)
}

// Warnings: Vàng hổ phách ấm, thêm Bold để chữ không bị chìm
pub fn warning_style() -> Style {
    Style::default()
        .fg(Palette::WARNING_AMBER)
        .add_modifier(Modifier::BOLD)
}

// Success: Xanh ngọc mint dịu mắt
pub fn success_style() -> Style {
    Style::default()
        .fg(Palette::SUCCESS_MINT)
}

pub enum Status {
    Success,
    Warning,
    Error,
}

pub fn style_for_status(status: Status) -> ratatui::style::Style {
    match status {
        Status::Success => success_style(),
        Status::Warning => warning_style(),
        Status::Error => error_style(),
    }
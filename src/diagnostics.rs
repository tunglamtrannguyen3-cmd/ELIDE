use crate::colors::{error_style, hint_style, warning_style, Palette};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub line: usize,
    pub message: String,
    pub severity: DiagnosticSeverity,
}

#[derive(Default)]
pub struct DiagnosticStore {
    pub diagnostics: Vec<Diagnostic>,
    pub is_visible: bool,
}

impl DiagnosticStore {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
            is_visible: false,
        }
    }

    pub fn toggle(&mut self) {
        self.is_visible = !self.is_visible;
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.is_visible {
            return;
        }

        let lines: Vec<Line> = if self.diagnostics.is_empty() {
            vec![Line::from(Span::styled(
                "No errors or warnings found!",
                Style::default().fg(Palette::SUCCESS_LIME),
            ))]
        } else {
            self.diagnostics
                .iter()
                .map(|diag| {
                    let (prefix, style) = match diag.severity {
                        DiagnosticSeverity::Error => ("[ERROR]", error_style()),
                        DiagnosticSeverity::Warning => ("[WARN]", warning_style()),
                        DiagnosticSeverity::Info => ("[INFO]", Style::default().fg(Color::Cyan)),
                        DiagnosticSeverity::Hint => ("[HINT]", hint_style()),
                    };

                    Line::from(vec![
                        Span::styled(format!("Line {:<4} ", diag.line), Style::default().fg(Palette::NAVY_GRAY)),
                        Span::styled(prefix, style),
                        Span::raw(format!(" {}", diag.message)),
                    ])
                })
                .collect()
        };

        let widget = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" LSP Diagnostics Overlay (Alt+L to close) "),
            );

        frame.render_widget(widget, area);
    }
}

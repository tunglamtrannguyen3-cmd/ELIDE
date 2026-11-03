// src/main.rs
#![warn(clippy::all, clippy::perf)]

mod editor;
mod palette;
mod lsp;
mod process;
mod colors;
mod ai;
mod clipboard;
mod git;

use anyhow::Result;
use colors::{palette_for, Theme};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use editor::Editor;
use lsp::{Diagnostic, LspClient};
use palette::Palette;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Terminal,
};
use std::{
    collections::HashMap,
    io::stdout,
    time::Duration,
};
use tokio::sync::mpsc;
use process::{TerminalGuard, KernelTracer};

fn to_ratatui_color(c: crossterm::style::Color) -> Color {
    match c {
        crossterm::style::Color::Rgb { r, g, b } => Color::Rgb(r, g, b),
        crossterm::style::Color::Black => Color::Black,
        crossterm::style::Color::DarkGrey => Color::DarkGray,
        crossterm::style::Color::Red => Color::Red,
        crossterm::style::Color::DarkRed => Color::Indexed(52),
        crossterm::style::Color::Green => Color::Green,
        crossterm::style::Color::DarkGreen => Color::Indexed(22),
        crossterm::style::Color::Yellow => Color::Yellow,
        crossterm::style::Color::DarkYellow => Color::Indexed(136),
        crossterm::style::Color::Blue => Color::Blue,
        crossterm::style::Color::DarkBlue => Color::Indexed(18),
        crossterm::style::Color::Magenta => Color::Magenta,
        crossterm::style::Color::DarkMagenta => Color::Indexed(90),
        crossterm::style::Color::Cyan => Color::Cyan,
        crossterm::style::Color::DarkCyan => Color::Indexed(30),
        crossterm::style::Color::White => Color::White,
        crossterm::style::Color::Grey => Color::Gray,
        _ => Color::Reset,
    }
}

struct AppState {
    editor: Editor,
    palette: Palette,
    lsp_client: LspClient,
    current_diagnostics: HashMap<String, Vec<Diagnostic>>,
    command_history: Vec<(String, bool)>,
    clipboard: String,
    doc_version: u32,
    buffer_cache: HashMap<String, Vec<String>>,
    pub should_exit: bool,
    show_lsp_pane: bool,
    pub show_ai_chat: bool,
    pub ai_input_buffer: String,
    pub ai_messages: Vec<(String, String)>,
    pub show_symbol_finder: bool,
    pub symbol_query: String,
    pub symbol_selected_idx: usize,
    pub current_theme: Theme,
}

impl AppState {
    fn new() -> Self {
        Self {
            editor: Editor::new(),
            palette: Palette::new(),
            lsp_client: LspClient::new(),
            current_diagnostics: HashMap::new(),
            command_history: Vec::new(),
            clipboard: String::new(),
            doc_version: 1,
            buffer_cache: HashMap::new(),
            should_exit: false,
            show_lsp_pane: false,
            show_ai_chat: false,
            ai_input_buffer: String::new(),
            ai_messages: Vec::new(),
            show_symbol_finder: false,
            symbol_query: String::new(),
            symbol_selected_idx: 0,
            current_theme: Theme::Serenity,
        }
    }

    fn normalize_path(path_str: &str) -> String {
        std::fs::canonicalize(path_str).map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|_| path_str.to_string())
    }

    fn cache_current_buffer(&mut self) {
        if let Some(ref filename) = self.editor.filename {
            let key = Self::normalize_path(filename);
            self.buffer_cache.insert(key, self.editor.lines.clone());
        }
    }

    fn open_buffered_file(&mut self, path_str: &str) -> Result<()> {
        self.cache_current_buffer();
        let canonical_path = Self::normalize_path(path_str);
        
        // Notify LSP client about the opened file (2 arguments)
        let full_text = std::fs::read_to_string(&canonical_path).unwrap_or_default();
        let _ = self.lsp_client.open_file(&canonical_path, &full_text);

        if let Some(cached_lines) = self.buffer_cache.get(&canonical_path) {
            self.editor.lines = cached_lines.clone();
            self.editor.filename = Some(canonical_path);
            self.editor.cursor = editor::Cursor { row: 0, col: 0 };
            self.editor.is_dirty = true;
        } else {
            self.editor.open(path_str)?;
            self.buffer_cache.insert(canonical_path.clone(), self.editor.lines.clone());
            self.editor.filename = Some(canonical_path);
        }
        Ok(())
    }

    fn new_buffered_file(&mut self, path_str: &str) -> Result<()> {
        self.cache_current_buffer();
        let canonical_path = Self::normalize_path(path_str);
        self.editor.filename = Some(canonical_path.clone());
        self.editor.lines = vec![String::new()];
        self.editor.cursor = editor::Cursor { row: 0, col: 0 };
        self.editor.is_dirty = true;
        self.buffer_cache.insert(canonical_path, self.editor.lines.clone());
        Ok(())
    }

    fn save_all_buffers(&mut self) -> Result<String, String> {
        self.cache_current_buffer();
        if self.buffer_cache.is_empty() {
            return match self.editor.save() {
                Ok(msg) => Ok(msg),
                Err(e) => Err(e.to_string()),
            };
        }
        let mut saved_count = 0;
        for (path_str, lines) in &self.buffer_cache {
            let mut content = lines.join("\n");
            content.push('\n');
            std::fs::write(path_str, content).map_err(|e| format!("Failed to save {}: {}", path_str, e))?;
            saved_count += 1;
        }
        self.editor.is_dirty = false;
        Ok(format!("Successfully saved {} buffer(s) to disk.", saved_count))
    }

    async fn handle_input(&mut self, key: crossterm::event::KeyEvent, diag_tx: &mpsc::UnboundedSender<(String, Vec<Diagnostic>)>) -> bool {
        if process::is_esc(&key) {
            if self.show_symbol_finder {
                self.show_symbol_finder = false;
            } else if self.show_ai_chat {
                self.show_ai_chat = false;
            } else if self.palette.is_active {
                self.palette.is_active = false;
            }
        } else if key.modifiers.contains(KeyModifiers::ALT) && key.code == KeyCode::Char('d') {
            self.show_symbol_finder = !self.show_symbol_finder;
            self.symbol_query.clear();
            self.symbol_selected_idx = 0;
        } else if self.show_symbol_finder {
            self.handle_symbol_finder_input(key);
        } else if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('a') {
            self.show_ai_chat = !self.show_ai_chat;
        } else if self.show_ai_chat {
            self.handle_ai_chat_input(key).await;
        } else if process::is_alt_t(&key) {
            self.palette.is_active = !self.palette.is_active;
        } else if process::is_alt_l(&key) {
            self.show_lsp_pane = !self.show_lsp_pane;
        } else if self.palette.is_active {
            self.handle_terminal_input(key, diag_tx).await;
        } else {
            self.handle_editor_input(key);
        }
        true
    }

    fn handle_symbol_finder_input(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => self.show_symbol_finder = false,
            KeyCode::Char(c) => {
                self.symbol_query.push(c);
                self.symbol_selected_idx = 0;
            }
            KeyCode::Backspace => {
                self.symbol_query.pop();
                self.symbol_selected_idx = 0;
            }
            KeyCode::Up => {
                self.symbol_selected_idx = self.symbol_selected_idx.saturating_sub(1);
            }
            KeyCode::Down => {
                let max_matches = self.get_filtered_symbols().len();
                if max_matches > 0 && self.symbol_selected_idx + 1 < max_matches {
                    self.symbol_selected_idx += 1;
                }
            }
            KeyCode::Enter => {
                let matches = self.get_filtered_symbols();
                if let Some((line_idx, _)) = matches.get(self.symbol_selected_idx) {
                    self.editor.cursor.row = *line_idx;
                    self.editor.cursor.col = 0;
                }
                self.show_symbol_finder = false;
            }
            _ => {}
        }
    }

    fn get_filtered_symbols(&self) -> Vec<(usize, String)> {
        self.editor.lines.iter().enumerate().filter_map(|(idx, line)| {
            let trimmed = line.trim();
            if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") || trimmed.starts_with("async fn ") {
                let func_name = trimmed.trim_end_matches('{').trim();
                if self.symbol_query.is_empty() || func_name.to_lowercase().contains(&self.symbol_query.to_lowercase()) {
                    return Some((idx, format!("L{}: {}", idx + 1, func_name)));
                }
            }
            None
        }).collect()
    }

    async fn handle_ai_chat_input(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                let prompt = self.ai_input_buffer.trim().to_string();
                self.ai_input_buffer.clear();
                if !prompt.is_empty() {
                    self.ai_messages.push(("user".to_string(), prompt));
                    let root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                    let messages = self.ai_messages.clone();
                    match ai::run_chat_stream(&root, messages).await {
                        Ok(res) => self.ai_messages.push(("assistant".to_string(), res)),
                        Err(e) => self.ai_messages.push(("assistant".to_string(), format!("Error: {}", e))),
                    }
                }
            }
            KeyCode::Char(c) => self.ai_input_buffer.push(c),
            KeyCode::Backspace => { self.ai_input_buffer.pop(); }
            _ => {}
        }
    }

    async fn handle_terminal_input(&mut self, key: crossterm::event::KeyEvent, diag_tx: &mpsc::UnboundedSender<(String, Vec<Diagnostic>)>) {
        match key.code {
            KeyCode::Enter => {
                let raw_input = self.palette.input_buffer.trim().to_string();

                if let Some(theme_name) = raw_input.strip_prefix("theme:") {
                    let t = match theme_name.trim().to_lowercase().as_str() {
                        "shrine" => Theme::SpringAtTheShrine,
                        "tokyo" => Theme::TokyoNight,
                        "firefly" => Theme::GraveyardOfTheFirefly,
                        "ocean" => Theme::DeepOcean,
                        "forest" => Theme::MagicForest,
                        "swamp" => Theme::Swampy,
                        "harvest" => Theme::HarvestTime,
                        _ => Theme::Serenity,
                    };
                    self.current_theme = t;
                    self.command_history.push((format!("Theme switched to {:?}", t), true));
                    self.palette.input_buffer.clear();
                    return;
                }

                let action = self.palette.parse_command(self.editor.filename.as_deref());
                match action {
                    palette::PaletteAction::Quit => {
                        self.should_exit = true;
                        self.palette.input_buffer.clear();
                        return;
                    }
                    palette::PaletteAction::SaveAndQuit => {
                        let _ = self.save_all_buffers();
                        self.should_exit = true;
                        self.palette.input_buffer.clear();
                        return;
                    }
                    palette::PaletteAction::SaveAll => {
                        let msg = self.save_all_buffers().unwrap_or_else(|e| e);
                        self.command_history.push((msg, true));
                        self.palette.input_buffer.clear();
                        return;
                    }
                    palette::PaletteAction::New(ref filename) => {
                        if !filename.is_empty() {
                            let _ = self.new_buffered_file(filename);
                        }
                        self.palette.input_buffer.clear();
                        return;
                    }
                    palette::PaletteAction::Code(ref filename) | palette::PaletteAction::Switch(ref filename) => {
                        if !filename.is_empty() {
                            let _ = self.open_buffered_file(filename);
                        }
                        self.palette.input_buffer.clear();
                        return;
                    }
                    _ => {}
                }

                let lsp_start_cmd = match &action {
                    palette::PaletteAction::SetLsp(cmd) if !cmd.is_empty() => Some(cmd.clone()),
                    _ => None,
                };

                let (msg, success) = self.palette.execute_action(action, &mut self.editor, &mut self.clipboard).await;
                
                if let Some(cmd) = lsp_start_cmd {
                    let curr_dir = std::env::current_dir().unwrap_or_default().to_string_lossy().to_string();
                    let target_lang = if let Some(ref fname) = self.editor.filename {
                        lsp::get_language_id(fname)
                    } else {
                        lsp::detect_workspace_language(&curr_dir).unwrap_or("plaintext")
                    };
                    let _ = self.lsp_client.init_workspace(&cmd, target_lang, &curr_dir, diag_tx.clone()).await;
                    self.show_lsp_pane = true;
                }

                self.command_history.push((msg, success));
                self.palette.input_buffer.clear();
            }
            KeyCode::Char(c) => self.palette.input_buffer.push(c),
            KeyCode::Backspace => { self.palette.input_buffer.pop(); }
            _ => {}
        }
    }

    fn handle_editor_input(&mut self, key: crossterm::event::KeyEvent) {
        let mut text_changed = false;
        match key.code {
            KeyCode::Char(c) => { if self.editor.filename.is_some() { self.editor.insert_char(c); text_changed = true; } }
            KeyCode::Enter => { if self.editor.filename.is_some() { self.editor.insert_newline(); text_changed = true; } }
            KeyCode::Backspace => { if self.editor.filename.is_some() { self.editor.backspace(); text_changed = true; } }
            KeyCode::Delete => { if self.editor.filename.is_some() { self.editor.delete_char(); text_changed = true; } }
            KeyCode::Left => self.editor.move_cursor(0, -1),
            KeyCode::Right => self.editor.move_cursor(0, 1),
            KeyCode::Up => self.editor.move_cursor(-1, 0),
            KeyCode::Down => self.editor.move_cursor(1, 0),
            _ => {}
        }
        if text_changed {
            if let Some(ref filename) = self.editor.filename {
                self.doc_version += 1;
                let full_text = self.editor.lines.join("\n");
                let _ = self.lsp_client.notify_change(filename, self.doc_version, &full_text);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let _terminal_guard = TerminalGuard::init()?;
    let tracer = KernelTracer::init();
    tracer.log_event("ELIDE session started with Git integration.");

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut state = AppState::new();
    let (diag_tx, mut diag_rx) = mpsc::unbounded_channel::<(String, Vec<Diagnostic>)>();

    loop {
        let p = palette_for(state.current_theme);
        let bg_main = to_ratatui_color(p.bg_main);
        let bg_panel = to_ratatui_color(p.bg_panel);
        let border_col = to_ratatui_color(p.border);
        let text_col = to_ratatui_color(p.text);
        let comment_col = to_ratatui_color(p.comment);
        let func_col = to_ratatui_color(p.function);
        let success_col = to_ratatui_color(p.success);
        let error_col = to_ratatui_color(p.error);
        let warning_col = to_ratatui_color(p.warning);
        let hint_col = to_ratatui_color(p.hint);
        let accent_col = to_ratatui_color(p.accent);
        let type_col = to_ratatui_color(p.type_color);

        terminal.draw(|f| {
            let size = f.area();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1), // Top Banner
                    Constraint::Min(10),   // Code Editor Workspace
                    Constraint::Length(if state.palette.is_active || state.show_ai_chat { 12 } else { 1 }), // Terminal/Status Pane
                    Constraint::Length(1), // Command Prompt Bar
                ])
                .split(size);

            let banner_p = Paragraph::new(Span::styled(
                format!(" ELIDE v2.4.0 — Theme: {:?}", state.current_theme),
                Style::default().fg(func_col).bg(bg_main)
            ));
            f.render_widget(banner_p, chunks[0]);

            let workspace_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(if state.show_lsp_pane { 75 } else { 100 }),
                    Constraint::Percentage(if state.show_lsp_pane { 25 } else { 0 }),
                ])
                .split(chunks[1]);

            let editor_height = workspace_chunks[0].height.saturating_sub(2) as usize;
            let editor_width = workspace_chunks[0].width.saturating_sub(10) as usize;

            let visible_lines = state.editor.render_visible_lines(editor_width, editor_height, text_col, comment_col);

            let editor_block = Paragraph::new(visible_lines)
                .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(border_col)).title(" Editor "))
                .style(Style::default().bg(bg_panel));
            f.render_widget(editor_block, workspace_chunks[0]);

            if !state.palette.is_active && !state.show_ai_chat && !state.show_symbol_finder {
                let cursor_row = workspace_chunks[0].y + 1 + (state.editor.cursor.row.saturating_sub(state.editor.row_offset)) as u16;
                let visual_col = state.editor.visual_cursor_col().saturating_sub(state.editor.col_offset);
                let cursor_col = workspace_chunks[0].x + 7 + visual_col as u16;
                if cursor_row < workspace_chunks[0].y + workspace_chunks[0].height - 1 {
                    f.set_cursor_position(Position::new(cursor_col, cursor_row));
                }
            }

            if state.show_lsp_pane {
                let lsp_lines: Vec<Line> = if state.current_diagnostics.is_empty() {
                    vec![Line::from(Span::styled("Nominal (No Diagnostics)", Style::default().fg(success_col)))]
                } else {
                    state.current_diagnostics.iter().flat_map(|(uri, diags)| {
                        let filename = uri.rsplit('/').next().unwrap_or(uri);
                        diags.iter().map(move |d| {
                            let diag_color = match d.severity {
                                Some(1) => error_col,
                                Some(2) => warning_col,
                                _ => hint_col,
                            };
                            Line::from(Span::styled(format!("{}:L{}: {}", filename, d.line, d.message), Style::default().fg(diag_color)))
                        })
                    }).collect()
                };
                let lsp_block = Paragraph::new(lsp_lines)
                    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(border_col)).title(" LSP Workspace "))
                    .style(Style::default().bg(bg_panel));
                f.render_widget(lsp_block, workspace_chunks[1]);
            }

            if state.palette.is_active {
                let term_lines: Vec<Line> = state.command_history.iter().map(|(msg, _)| Line::from(msg.clone())).collect();
                let term_block = Paragraph::new(term_lines)
                    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(accent_col)).title(" Terminal Output "))
                    .style(Style::default().bg(bg_panel));
                f.render_widget(term_block, chunks[2]);
            } else if state.show_ai_chat {
                let ai_lines: Vec<Line> = state.ai_messages.iter().map(|(role, msg)| {
                    Line::from(format!("{}: {}", role, msg))
                }).collect();
                let ai_block = Paragraph::new(ai_lines)
                    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(hint_col)).title(" AI Auditor Chat "))
                    .style(Style::default().bg(bg_panel));
                f.render_widget(ai_block, chunks[2]);
            } else {
                let file_status = if let Some((_, success)) = state.command_history.last() {
                    if !success {
                        colors::Status::Error
                    } else if state.editor.is_dirty {
                        colors::Status::Modified
                    } else {
                        colors::Status::Saved
                    }
                } else if state.editor.is_dirty {
                    colors::Status::Modified
                } else {
                    colors::Status::Saved
                };
                let status_color = to_ratatui_color(colors::color_for_status_in(state.current_theme, file_status));

                let is_bold = matches!(colors::attribute_for_status(file_status), crossterm::style::Attribute::Bold);
                let mut status_style = Style::default().fg(status_color);
                if is_bold {
                    status_style = status_style.add_modifier(Modifier::BOLD);
                }

                let status_text = vec![
                    Span::styled(format!(" File: {} ", state.editor.filename.as_deref().unwrap_or("[No File]")), Style::default().fg(text_col)),
                    Span::styled(format!(" [{:?}] ", file_status), status_style),
                    Span::styled(format!("| R:{} C:{} ", state.editor.cursor.row + 1, state.editor.cursor.col + 1), Style::default().fg(text_col)),
                ];
                let status_p = Paragraph::new(Line::from(status_text));
                f.render_widget(status_p, chunks[2]);
            }

            let prompt_str = if state.palette.is_active {
                format!(" : {}", state.palette.input_buffer)
            } else {
                format!(" : (Ctrl+A AI, Alt+T Term, Alt+D Symbols) [{:?}]", state.current_theme)
            };
            let cmd_p = Paragraph::new(prompt_str).style(Style::default().fg(type_col));
            f.render_widget(cmd_p, chunks[3]);

            if state.show_symbol_finder {
                let area = centered_rect(60, 40, size);
                f.render_widget(Clear, area);
                let matches = state.get_filtered_symbols();
                let match_lines: Vec<Line> = matches.iter().map(|(_, l)| Line::from(l.clone())).collect();
                let modal = Paragraph::new(match_lines)
                    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(accent_col)).title(" 🔍 Function Symbol Finder "))
                    .style(Style::default().bg(bg_panel));
                f.render_widget(modal, area);
            }
        })?;

        while let Ok((uri, diags)) = diag_rx.try_recv() {
            if diags.is_empty() {
                state.current_diagnostics.remove(&uri);
            } else {
                state.current_diagnostics.insert(uri, diags);
            }
        }

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if !state.handle_input(key, &diag_tx).await {
                    tracer.log_event("Exit requested via key handler.");
                    break;
                }
            }
        }

        if state.should_exit {
            tracer.log_event("Exit requested via state flag.");
            break;
        }
    }

    Ok(())
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
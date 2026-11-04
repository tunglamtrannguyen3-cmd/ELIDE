// src/main.rs
#![warn(clippy::all, clippy::perf)]

mod editor;
mod palette;
mod lsp;
mod process;
mod colors;
mod ai;
mod git;

use anyhow::Result;
use colors::{palette_for, Theme};
use crossterm::event::{KeyCode, KeyModifiers};
use editor::{Editor, SystemClipboard};
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
use process::{TerminalGuard, KernelTracer, InputEvent};

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
    pub file_tree: process::FileTree,
    pub ai_tx: mpsc::UnboundedSender<Result<String, String>>,
    pub ai_rx: mpsc::UnboundedReceiver<Result<String, String>>,
    pub is_waiting_for_ai: bool,
}

impl AppState {
    fn new() -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let (ai_tx, ai_rx) = mpsc::unbounded_channel();
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
            file_tree: process::FileTree::new(current_dir),
            ai_tx,
            ai_rx,
            is_waiting_for_ai: false,
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
        
        let full_text = std::fs::read_to_string(&canonical_path).unwrap_or_default();
        let _ = self.lsp_client.open_file(&canonical_path, &full_text);

        if let Some(cached_lines) = self.buffer_cache.get(&canonical_path) {
            self.editor.lines = cached_lines.clone();
            self.editor.filename = Some(canonical_path.clone());
            self.editor.cursor = editor::Cursor { row: 0, col: 0 };
            self.editor.is_dirty = false; // Fixed: Cached pristine files start clean[cite: 3]
        } else {
            self.editor.open(&canonical_path)?;
            self.buffer_cache.insert(canonical_path.clone(), self.editor.lines.clone());
            self.editor.filename = Some(canonical_path.clone());
        }

        self.doc_version += 1;
        let active_text = self.editor.lines.join("\n");
        let _ = self.lsp_client.notify_change(&canonical_path, self.doc_version, &active_text);

        Ok(())
    }

    fn new_buffered_file(&mut self, path_str: &str) -> Result<()> {
        self.cache_current_buffer();
        let canonical_path = Self::normalize_path(path_str);
        self.editor.filename = Some(canonical_path.clone());
        self.editor.lines = vec![String::new()];
        self.editor.cursor = editor::Cursor { row: 0, col: 0 };
        self.editor.is_dirty = true;
        self.buffer_cache.insert(canonical_path.clone(), self.editor.lines.clone());

        self.doc_version += 1;
        let _ = self.lsp_client.open_file(&canonical_path, "");
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

    async fn handle_key_input(&mut self, key: crossterm::event::KeyEvent, diag_tx: &mpsc::UnboundedSender<(String, Vec<Diagnostic>)>) -> bool {
        // Fixed: Restricted AI hotkey strictly to Alt+A to preserve Ctrl+A text editing behavior[cite: 3]
        let is_ai_hotkey = key.modifiers.contains(KeyModifiers::ALT) && key.code == KeyCode::Char('a');

        if process::is_esc(&key) {
            if self.file_tree.is_visible {
                self.file_tree.is_visible = false;
            } else if self.show_symbol_finder {
                self.show_symbol_finder = false;
            } else if self.show_ai_chat {
                self.show_ai_chat = false;
            } else if self.palette.is_active {
                self.palette.is_active = false;
            }
        } else if self.file_tree.is_visible {
            match key.code {
                KeyCode::Up => self.file_tree.move_selection(-1),
                KeyCode::Down => self.file_tree.move_selection(1),
                KeyCode::Enter => {
                    if let Some(path) = self.file_tree.toggle_selected() {
                        let path_str = path.to_string_lossy().to_string();
                        let _ = self.open_buffered_file(&path_str);
                        self.file_tree.is_visible = false;
                    }
                }
                KeyCode::Char('q') | KeyCode::Char('t') => self.file_tree.is_visible = false,
                _ => {}
            }
        } else if key.modifiers.contains(KeyModifiers::ALT) && key.code == KeyCode::Char('d') {
            self.show_symbol_finder = !self.show_symbol_finder;
            self.symbol_query.clear();
            self.symbol_selected_idx = 0;
        } else if self.show_symbol_finder {
            self.handle_symbol_finder_input(key);
        } else if is_ai_hotkey {
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

    fn handle_paste_input(&mut self, pasted_text: &str) {
        if self.palette.is_active {
            self.palette.input_buffer.push_str(pasted_text);
        } else if self.show_ai_chat {
            self.ai_input_buffer.push_str(pasted_text);
        } else if self.show_symbol_finder {
            self.symbol_query.push_str(pasted_text);
        } else if self.editor.filename.is_some() {
            for c in pasted_text.chars() {
                if c == '\n' {
                    self.editor.insert_newline();
                } else {
                    self.editor.insert_char(c);
                }
            }
            if let Some(ref filename) = self.editor.filename {
                self.doc_version += 1;
                let full_text = self.editor.lines.join("\n");
                let _ = self.lsp_client.notify_change(filename, self.doc_version, &full_text);
            }
        }
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

    fn is_function_line(trimmed: &str) -> bool {
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            return false;
        }
        trimmed.contains("fn ") && (
            trimmed.starts_with("fn ") ||
            trimmed.starts_with("pub ") ||
            trimmed.starts_with("async ") ||
            trimmed.starts_with("unsafe ") ||
            trimmed.starts_with("const ") ||
            trimmed.starts_with("extern ") ||
            trimmed.starts_with("default ")
        )
    }

    fn get_filtered_symbols(&self) -> Vec<(usize, String)> {
        self.editor.lines.iter().enumerate().filter_map(|(idx, line)| {
            let trimmed = line.trim();
            if Self::is_function_line(trimmed) {
                let func_name = trimmed.trim_end_matches('{').trim();
                if self.symbol_query.is_empty() || func_name.to_lowercase().contains(&self.symbol_query.to_lowercase()) {
                    return Some((idx, format!("L{}: {}", idx + 1, func_name)));
                }
            }
            None
        }).collect()
    }

    async fn handle_ai_chat_input(&mut self, key: crossterm::event::KeyEvent) {
        if self.is_waiting_for_ai {
            return; // Ignore all input while waiting for active AI response
        }

        match key.code {
            KeyCode::Enter => {
                let prompt = self.ai_input_buffer.trim().to_string();
                self.ai_input_buffer.clear();
                if !prompt.is_empty() {
                    self.is_waiting_for_ai = true;
                    self.ai_messages.push(("user".to_string(), prompt));
                    self.ai_messages.push(("assistant".to_string(), "🤖 AI is thinking...".to_string()));
                    let root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                    let messages = self.ai_messages.clone();
                    let tx = self.ai_tx.clone();
                    tokio::spawn(async move {
                        let res = ai::run_chat_stream(&root, messages).await;
                        let _ = tx.send(res);
                    });
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
                    let t = Theme::from_input(theme_name);
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
                        match self.save_all_buffers() {
                            Ok(msg) => {
                                self.command_history.push((msg, true));
                                self.should_exit = true;
                            }
                            Err(e) => {
                                self.command_history.push((format!("Save Error (Abort Exit): {}", e), false));
                            }
                        }
                        self.palette.input_buffer.clear();
                        return;
                    }
                    palette::PaletteAction::SaveAll => {
                        match self.save_all_buffers() {
                            Ok(msg) => self.command_history.push((msg, true)),
                            Err(e) => self.command_history.push((format!("Save All Error: {}", e), false)),
                        }
                        self.palette.input_buffer.clear();
                        return;
                    }
                    palette::PaletteAction::Tree => {
                        self.file_tree.refresh();
                        self.file_tree.is_visible = !self.file_tree.is_visible;
                        self.palette.input_buffer.clear();
                        return;
                    }
                    palette::PaletteAction::CopyTerm => {
                        let term_text: String = self.command_history.iter().map(|(msg, _)| msg.clone()).collect::<Vec<_>>().join("\n");
                        self.clipboard = term_text.clone();
                        let mut clip_mgr = SystemClipboard::new();
                        let (msg, success) = match clip_mgr.set_text(&term_text) {
                            Ok(_) => ("Copied full terminal history to system clipboard & internal buffer.".to_string(), true),
                            Err(e) => (format!("Copied internally, but clipboard sync warned: {}", e), false),
                        };
                        self.command_history.push((msg, success));
                        self.palette.input_buffer.clear();
                        return;
                    }
                    palette::PaletteAction::CopyLsp => {
                        let lsp_text: String = self.current_diagnostics.iter().flat_map(|(uri, diags)| {
                            let filename = uri.rsplit('/').next().unwrap_or(uri);
                            diags.iter().map(move |d| format!("{}:L{}: {}", filename, d.line, d.message))
                        }).collect::<Vec<_>>().join("\n");
                        self.clipboard = lsp_text.clone();
                        let mut clip_mgr = SystemClipboard::new();
                        let (msg, success) = match clip_mgr.set_text(&lsp_text) {
                            Ok(_) => ("Copied active LSP diagnostics to system clipboard & internal buffer.".to_string(), true),
                            Err(e) => (format!("Copied internally, but clipboard sync warned: {}", e), false),
                        };
                        self.command_history.push((msg, success));
                        self.palette.input_buffer.clear();
                        return;
                    }
                    palette::PaletteAction::New(ref filename) => {
                        if !filename.is_empty() {
                            match self.new_buffered_file(filename) {
                                Ok(_) => self.command_history.push((format!("Created new file: {}", filename), true)),
                                Err(e) => self.command_history.push((format!("Error creating file: {}", e), false)),
                            }
                        }
                        self.palette.input_buffer.clear();
                        return;
                    }
                    palette::PaletteAction::Code(ref filename) | palette::PaletteAction::Switch(ref filename) => {
                        if !filename.is_empty() {
                            match self.open_buffered_file(filename) {
                                Ok(_) => self.command_history.push((format!("Opened file: {}", filename), true)),
                                Err(e) => self.command_history.push((format!("Error opening file: {}", e), false)),
                            }
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
            KeyCode::Tab => { if self.editor.filename.is_some() { self.editor.insert_tab(); text_changed = true; } }
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
    tracer.log_event("ELIDE session started with fully audited process & state integration.");

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

            let term_pane_height = if state.palette.is_active || state.show_ai_chat {
                std::cmp::min(12, size.height.saturating_sub(3))
            } else {
                1
            };

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Min(3),
                    Constraint::Length(term_pane_height),
                    Constraint::Length(1),
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
                    Constraint::Percentage(if state.file_tree.is_visible { 22 } else { 0 }),
                    Constraint::Percentage(if state.file_tree.is_visible { 78 } else { if state.show_lsp_pane { 75 } else { 100 } }),
                    Constraint::Percentage(if state.file_tree.is_visible { 0 } else { if state.show_lsp_pane { 25 } else { 0 } }),
                ])
                .split(chunks[1]);

            let editor_workspace_idx = if state.file_tree.is_visible {
                let tree_lines: Vec<Line> = state.file_tree.nodes.iter().enumerate().map(|(idx, node)| {
                    let indent = "  ".repeat(node.depth);
                    let prefix = if node.is_dir {
                        if state.file_tree.expanded_dirs.contains(&node.path) { "📂 " } else { "📁 " }
                    } else {
                        "📄 "
                    };
                    let display_str = format!("{}{}{}", indent, prefix, node.name);
                    let line_style = if idx == state.file_tree.selected_idx {
                        Style::default().fg(accent_col).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(text_col)
                    };
                    Line::from(Span::styled(display_str, line_style))
                }).collect();

                let tree_block = Paragraph::new(tree_lines)
                    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(border_col)).title(" 📂 Workspace "))
                    .style(Style::default().bg(bg_panel));
                f.render_widget(tree_block, workspace_chunks[0]);
                1
            } else {
                0
            };

            let editor_chunk = workspace_chunks[editor_workspace_idx];
            let editor_height = editor_chunk.height.saturating_sub(2) as usize;
            let editor_width = editor_chunk.width.saturating_sub(10) as usize;

            let visible_lines = state.editor.render_visible_lines(editor_width, editor_height, text_col, comment_col);

            let editor_block = Paragraph::new(visible_lines)
                .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(border_col)).title(" Editor "))
                .style(Style::default().bg(bg_panel));
            f.render_widget(editor_block, editor_chunk);

            if !state.palette.is_active && !state.show_ai_chat && !state.show_symbol_finder && !state.file_tree.is_visible {
                let cursor_row = editor_chunk.y + 1 + (state.editor.cursor.row.saturating_sub(state.editor.row_offset)) as u16;
                let visual_col = state.editor.visual_cursor_col().saturating_sub(state.editor.col_offset);
                let raw_cursor_col = editor_chunk.x + 7 + visual_col as u16;
                
                let max_col = editor_chunk.x + editor_chunk.width.saturating_sub(2);
                let cursor_col = std::cmp::min(raw_cursor_col, max_col);

                if cursor_row < editor_chunk.y + editor_chunk.height.saturating_sub(1) {
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
                f.render_widget(lsp_block, workspace_chunks[2]);
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
                format!(" : (Alt+A AI, Alt+T Term, Alt+D Symbols) [{:?}]", state.current_theme)
            };
            let cmd_p = Paragraph::new(prompt_str).style(Style::default().fg(type_col));
            f.render_widget(cmd_p, chunks[3]);

            if state.show_symbol_finder {
                let area = centered_rect(65, 45, size);
                f.render_widget(Clear, area);
                let matches = state.get_filtered_symbols();
                
                let mut symbol_lines = vec![
                    Line::from(Span::styled(format!("Query: {}_", state.symbol_query), Style::default().fg(accent_col).add_modifier(Modifier::BOLD))),
                    Line::from(""),
                ];
                
                if matches.is_empty() {
                    symbol_lines.push(Line::from(Span::styled("No matching function symbols found.", Style::default().fg(comment_col))));
                } else {
                    for (idx, (_, label)) in matches.iter().enumerate() {
                        let line_style = if idx == state.symbol_selected_idx {
                            Style::default().fg(accent_col).add_modifier(Modifier::BOLD | Modifier::REVERSED)
                        } else {
                            Style::default().fg(text_col)
                        };
                        symbol_lines.push(Line::from(Span::styled(format!("  {} ", label), line_style)));
                    }
                }

                let modal = Paragraph::new(symbol_lines)
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

        while let Ok(res) = state.ai_rx.try_recv() {
            state.is_waiting_for_ai = false;
            if let Some((role, _)) = state.ai_messages.last() {
                if role == "assistant" && state.ai_messages.last().unwrap().1.contains("thinking") {
                    state.ai_messages.pop();
                }
            }
            match res {
                Ok(content) => state.ai_messages.push(("assistant".to_string(), content)),
                Err(e) => state.ai_messages.push(("assistant".to_string(), format!("Error: {}", e))),
            }
        }

        match process::poll_event(Duration::from_millis(16))? {
            InputEvent::Key(key) => {
                if !state.handle_key_input(key, &diag_tx).await {
                    tracer.log_event("Exit requested via key handler.");
                    break;
                }
            }
            InputEvent::Paste(pasted_text) => {
                state.handle_paste_input(&pasted_text);
            }
            InputEvent::ScrollUp => {
                state.editor.move_cursor(-3, 0);
            }
            InputEvent::ScrollDown => {
                state.editor.move_cursor(3, 0);
            }
            InputEvent::Resize | InputEvent::Tick => {}
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
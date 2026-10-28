// src/main.rs
#![warn(clippy::all, clippy::perf)]

mod editor;
mod palette;
mod terminal;
mod lsp;
mod diagnostics;
mod process;
mod colors;
mod tracer;
mod lsp_writer;
mod ai;
mod clipboard;

use anyhow::Result;
use crossterm::{
    cursor,
    event::KeyCode,
    style::{Attribute, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use editor::Editor;
use lsp::LspClient;
use palette::{Palette, PaletteAction};
use std::{
    collections::HashMap,
    io::{stdout, Stdout, Write},
    path::Path,
    time::Duration,
};
use tokio::sync::mpsc;
use terminal::InputEvent;

fn flush_token(stdout: &mut Stdout, word: &mut String, next_char: Option<char>) -> std::io::Result<()> {
    if word.is_empty() {
        return Ok(());
    }
    let color = match word.as_str() {
        "fn" | "let" | "mut" | "return" | "if" | "else" | "match" | "struct" | "enum" | "pub" | "use" | "impl" | "for" | "while" | "loop" | "const" | "static" | "async" | "await" | "trait" | "type" | "as" | "ref" | "mod" | "where" | "dyn" | "move" | "unsafe" | "extern" | "break" | "continue" => colors::Palette::KEYWORD_MAUVE,
        "String" | "usize" | "isize" | "i8" | "i16" | "i32" | "i64" | "i128" | "u8" | "u16" | "u32" | "u64" | "u128" | "f32" | "f64" | "bool" | "char" | "Vec" | "Option" | "Result" | "Self" | "Box" | "Rc" | "Arc" | "HashMap" | "BTreeMap" => colors::Palette::TYPE_YELLOW,
        "true" | "false" | "Some" | "None" | "Ok" | "Err" => colors::Palette::NUMBER_PEACH,
        _ => {
            if word.starts_with('\'') {
                colors::Palette::STRING_GREEN
            } else if word.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '_') || word.starts_with("0x") {
                colors::Palette::NUMBER_PEACH
            } else if next_char == Some('(') || next_char == Some('!') {
                colors::Palette::FUNCTION_BLUE
            } else {
                colors::Palette::TEXT_DEFAULT
            }
        }
    };
    stdout.execute(SetForegroundColor(color))?;
    write!(stdout, "{}", word)?;
    word.clear();
    Ok(())
}

struct AppState {
    editor: Editor,
    palette: Palette,
    lsp_client: LspClient,
    current_diagnostics: HashMap<String, Vec<diagnostics::Diagnostic>>,
    command_history: Vec<(String, bool)>,
    clipboard: String,
    doc_version: u32,
    buffer_cache: HashMap<String, Vec<String>>,
    pub should_exit: bool,
    show_lsp_pane: bool,
    lsp_scroll_offset: usize,
    term_scroll_y: usize,
    term_scroll_x: usize,
    pub show_ai_chat: bool,
    pub ai_input_buffer: String,
    pub ai_messages: Vec<(String, String)>,
    pub ai_scroll_offset: usize,
    pub show_symbol_finder: bool,
    pub symbol_query: String,
    pub symbol_selected_idx: usize,
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
            lsp_scroll_offset: 0,
            term_scroll_y: 0,
            term_scroll_x: 0,
            show_ai_chat: false,
            ai_input_buffer: String::new(),
            ai_messages: Vec::new(),
            ai_scroll_offset: 0,
            show_symbol_finder: false,
            symbol_query: String::new(),
            symbol_selected_idx: 0,
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
        if let Some(cached_lines) = self.buffer_cache.get(&canonical_path) {
            self.editor.lines = cached_lines.clone();
            self.editor.filename = Some(canonical_path);
            self.editor.cursor = editor::Cursor { row: 0, col: 0 };
            self.editor.row_offset = 0;
            self.editor.col_offset = 0;
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
        self.editor.row_offset = 0;
        self.editor.col_offset = 0;
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

    async fn handle_input(&mut self, key: crossterm::event::KeyEvent, diag_tx: &mpsc::UnboundedSender<(String, Vec<diagnostics::Diagnostic>)>) -> bool {
        if terminal::is_esc(&key) {
            if self.show_symbol_finder {
                self.show_symbol_finder = false;
            } else if self.show_ai_chat {
                self.show_ai_chat = false;
            } else if self.palette.is_active {
                self.palette.is_active = false;
            } else {
                return false;
            }
        } else if key.modifiers.contains(crossterm::event::KeyModifiers::ALT) && key.code == KeyCode::Char('d') {
            self.show_symbol_finder = !self.show_symbol_finder;
            self.symbol_query.clear();
            self.symbol_selected_idx = 0;
        } else if self.show_symbol_finder {
            self.handle_symbol_finder_input(key);
        } else if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) && key.code == KeyCode::Char('a') {
            self.show_ai_chat = !self.show_ai_chat;
        } else if self.show_ai_chat {
            self.handle_ai_chat_input(key).await;
        } else if terminal::is_alt_t(&key) {
            self.palette.is_active = !self.palette.is_active;
        } else if terminal::is_alt_l(&key) {
            self.show_lsp_pane = !self.show_lsp_pane;
        } else if terminal::is_alt_up(&key) {
            self.lsp_scroll_offset = self.lsp_scroll_offset.saturating_sub(1);
        } else if terminal::is_alt_down(&key) {
            self.lsp_scroll_offset = self.lsp_scroll_offset.saturating_add(1);
        } else if self.palette.is_active {
            self.handle_terminal_input(key, diag_tx).await;
        } else {
            self.handle_editor_input(key);
        }
        true
    }

    fn handle_symbol_finder_input(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => { self.show_symbol_finder = false; }
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
            if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") || trimmed.starts_with("async fn ") || trimmed.starts_with("pub async fn ") {
                let func_name = trimmed.trim_end_matches('{').trim();
                if self.symbol_query.is_empty() || func_name.to_lowercase().contains(&self.symbol_query.to_lowercase()) {
                    return Some((idx, format!("L{}: {}", idx+1, func_name)));
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
                    let mut full_prompt = prompt;
                    if !self.current_diagnostics.is_empty() {
                        full_prompt.push_str("\n\n--- Current Compiler / LSP Diagnostics Log ---");
                        let mut sorted_uris: Vec<_> = self.current_diagnostics.keys().collect();
                        sorted_uris.sort();
                        for uri in sorted_uris {
                            let diags = &self.current_diagnostics[uri];
                            let filename = uri.rsplit('/').next().unwrap_or(uri);
                            for diag in diags {
                                full_prompt.push_str(&format!("\n- {}:L{}: {}", filename, diag.line, diag.message));
                            }
                        }
                    }

                    self.ai_messages.push(("user".to_string(), full_prompt));
                    let root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                    let messages = self.ai_messages.clone();

                    match crate::ai::run_chat_stream(&root, messages).await {
                        Ok(response) => {
                            self.ai_messages.push(("assistant".to_string(), response));
                        }
                        Err(e) => {
                            self.ai_messages.push(("assistant".to_string(), format!("❌ AI Error: {}", e)));
                        }
                    }
                }
            }
            KeyCode::Char(c) => self.ai_input_buffer.push(c),
            KeyCode::Backspace => { self.ai_input_buffer.pop(); }
            KeyCode::Up => self.ai_scroll_offset = self.ai_scroll_offset.saturating_add(1),
            KeyCode::Down => self.ai_scroll_offset = self.ai_scroll_offset.saturating_sub(1),
            _ => {}
        }
    }

    async fn handle_terminal_input(&mut self, key: crossterm::event::KeyEvent, diag_tx: &mpsc::UnboundedSender<(String, Vec<diagnostics::Diagnostic>)>) {
        match key.code {
            KeyCode::Up => self.term_scroll_y = self.term_scroll_y.saturating_add(1),
            KeyCode::Down => self.term_scroll_y = self.term_scroll_y.saturating_sub(1),
            KeyCode::Left => self.term_scroll_x = self.term_scroll_x.saturating_sub(1),
            KeyCode::Right => self.term_scroll_x = self.term_scroll_x.saturating_add(1),
            KeyCode::Enter => {
                self.term_scroll_y = 0;
                self.term_scroll_x = 0;
                let action = self.palette.parse_command(self.editor.filename.as_deref());
                let lsp_start_cmd = match &action {
                    PaletteAction::SetLsp(cmd) if !cmd.is_empty() => Some(cmd.clone()),
                    _ => None,
                };
                let (msg, success) = match action.clone() {
                    PaletteAction::SaveAll => {
                        match self.save_all_buffers() {
                            Ok(m) => (m, true),
                            Err(e) => (format!("Save All Error: {}", e), false),
                        }
                    }
                    PaletteAction::Quit => {
                        self.should_exit = true;
                        ("Quitting without saving...".to_string(), true)
                    }
                    PaletteAction::SaveAndQuit => {
                        self.cache_current_buffer();
                        match self.save_all_buffers() {
                            Ok(m) => {
                                self.should_exit = true;
                                (format!("{} Quitting...", m), true)
                            }
                            Err(e) => (format!("Save Error: {}", e), false),
                        }
                    }
                    PaletteAction::Code(ref filename) | PaletteAction::Switch(ref filename) => {
                        if filename.is_empty() {
                            ("Error: Specify a file.".to_string(), false)
                        } else {
                            match self.open_buffered_file(filename) {
                                Ok(_) => (format!("Switched to file: {}", filename), true),
                                Err(e) => (format!("Failed to open {}: {}", filename, e), false),
                            }
                        }
                    }
                    PaletteAction::New(ref filename) => {
                        if filename.is_empty() {
                            ("Error: Filename cannot be empty.".to_string(), false)
                        } else {
                            match self.new_buffered_file(filename) {
                                Ok(_) => (format!("Created new buffer: {}", filename), true),
                                Err(e) => (format!("Failed to create {}: {}", filename, e), false),
                            }
                        }
                    }
                    PaletteAction::Tree => {
                        self.cache_current_buffer();
                        let curr_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                        match std::fs::read_dir(&curr_dir) {
                            Ok(entries) => {
                                let mut lines = vec![
                                    format!(" Workspace: {}", curr_dir.display()),
                                    " ──────────────────────────────────────────".to_string(),
                                    " ../".to_string(),
                                ];
                                let mut sorted_entries: Vec<String> = entries.flatten().filter_map(|e| {
                                    let name = e.file_name().to_string_lossy().to_string();
                                    if name.starts_with('.') || name == "target" || name == "node_modules" { return None; }
                                    if e.path().is_dir() { Some(format!("{}/", name)) } else { Some(name.to_string()) }
                                }).collect();
                                sorted_entries.sort();
                                lines.extend(sorted_entries);
                                self.editor.lines = lines;
                                self.editor.filename = None;
                                self.editor.cursor.row = 0;
                                self.editor.cursor.col = 0;
                                self.editor.row_offset = 0;
                                self.editor.col_offset = 0;
                                ("Returned to workspace file tree.".to_string(), true)
                            }
                            Err(e) => (format!("Failed to read workspace directory: {}", e), false),
                        }
                    }
                    PaletteAction::CopyTerm => {
                        let text_to_copy = self.command_history.iter().map(|(msg, _)| msg.clone()).collect::<Vec<String>>().join("\n");
                        self.clipboard = text_to_copy.clone();
                        let mut clip_mgr = clipboard::SystemClipboard::new();
                        match clip_mgr.set_text(&text_to_copy) {
                            Ok(_) => ("Copied terminal log to clipboard!".to_string(), true),
                            Err(e) => (format!("Copied internally, but clipboard sync warned: {}", e), false),
                        }
                    }
                    PaletteAction::CopyLsp => {
                        let mut wrapped_lines = Vec::new();
                        let mut sorted_uris: Vec<_> = self.current_diagnostics.keys().collect();
                        sorted_uris.sort();
                        for uri in sorted_uris {
                            let diags = &self.current_diagnostics[uri];
                            let filename = uri.rsplit('/').next().unwrap_or("?");
                            for diag in diags {
                                wrapped_lines.push(format!("{}:L{}: {}", filename, diag.line, diag.message));
                            }
                        }
                        let text_to_copy = if wrapped_lines.is_empty() {
                            "No LSP diagnostics to copy (Nominal).".to_string()
                        } else {
                            wrapped_lines.join("\n")
                        };
                        self.clipboard = text_to_copy.clone();
                        let mut clip_mgr = clipboard::SystemClipboard::new();
                        match clip_mgr.set_text(&text_to_copy) {
                            Ok(_) => ("Copied LSP diagnostics to clipboard!".to_string(), true),
                            Err(e) => (format!("Copied internally, but clipboard sync warned: {}", e), false),
                        }
                    }
                    _ => {
                        self.palette.execute_action(action, &mut self.editor, &mut self.clipboard).await
                    }
                };

                if let Some(cmd) = lsp_start_cmd {
                    let curr_dir = std::env::current_dir().unwrap_or_default().to_string_lossy().to_string();
                    let target_lang = if let Some(ref fname) = self.editor.filename {
                        lsp::get_language_id(fname)
                    } else {
                        lsp::detect_workspace_language(&curr_dir).unwrap_or("plaintext")
                    };
                    if let Err(e) = self.lsp_client.init_workspace(&cmd, target_lang, &curr_dir, diag_tx.clone()).await {
                        self.command_history.push((format!("LSP Boot Error: {}", e), false));
                    } else {
                        self.show_lsp_pane = true;
                        if let Some(ref filename) = self.editor.filename {
                            let text = self.editor.lines.join("\n");
                            let _ = self.lsp_client.open_file(filename, &text);
                        }
                    }
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
            KeyCode::Char(c) => {
                if self.editor.filename.is_some() {
                    self.editor.insert_char(c);
                    text_changed = true;
                }
            }
            KeyCode::Enter => {
                if self.editor.filename.is_none() {
                    self.open_selected_tree_item();
                } else {
                    self.editor.insert_newline();
                    text_changed = true;
                }
            }
            KeyCode::Backspace => {
                if self.editor.filename.is_some() {
                    self.editor.backspace();
                    text_changed = true;
                } else {
                    self.open_parent_directory();
                }
            }
            KeyCode::Delete => {
                if self.editor.filename.is_some() {
                    self.editor.delete_char();
                    text_changed = true;
                }
            }
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

    fn open_parent_directory(&mut self) {
        if self.editor.filename.is_none() {
            let parent_path = std::path::Path::new("..");
            if let Ok(canonical_dir) = std::fs::canonicalize(parent_path) {
                let _ = std::env::set_current_dir(&canonical_dir);
                if let Ok(entries) = std::fs::read_dir(&canonical_dir) {
                    let mut lines = vec![
                        format!(" Workspace: {}", canonical_dir.display()),
                        " ──────────────────────────────────────────".to_string(),
                        " ../".to_string(),
                    ];
                    let mut sorted_entries: Vec<String> = entries.flatten().filter_map(|e| {
                        let name = e.file_name().to_string_lossy().to_string();
                        if name.starts_with('.') || name == "target" || name == "node_modules" { return None; }
                        if e.path().is_dir() { Some(format!("{}/", name)) } else { Some(name.to_string()) }
                    }).collect();
                    sorted_entries.sort();
                    lines.extend(sorted_entries);
                    self.editor.lines = lines;
                    self.editor.cursor.row = 0;
                    self.editor.cursor.col = 0;
                    self.editor.row_offset = 0;
                    self.editor.col_offset = 0;
                }
            }
        }
    }

    fn open_selected_tree_item(&mut self) {
        let row = self.editor.cursor.row;
        if row >= self.editor.lines.len() { return; }
        let raw_line = &self.editor.lines[row];
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with("Workspace:") || trimmed.starts_with('─') { return; }
        let target_str = trimmed.trim_end_matches('/');
        let path = Path::new(target_str);
        if path.is_dir() {
            if let Ok(canonical_dir) = std::fs::canonicalize(path) {
                let _ = std::env::set_current_dir(&canonical_dir);
                if let Ok(entries) = std::fs::read_dir(&canonical_dir) {
                    let mut lines = vec![
                        format!(" Workspace: {}", canonical_dir.display()),
                        " ──────────────────────────────────────────".to_string(),
                        " ../".to_string(),
                    ];
                    let mut sorted_entries: Vec<String> = entries.flatten().filter_map(|e| {
                        let name = e.file_name().to_string_lossy().to_string();
                        if name.starts_with('.') || name == "target" || name == "node_modules" { return None; }
                        if e.path().is_dir() { Some(format!("{}/", name)) } else { Some(name.to_string()) }
                    }).collect();
                    sorted_entries.sort();
                    lines.extend(sorted_entries);
                    self.editor.lines = lines;
                    self.editor.cursor.row = 0;
                    self.editor.cursor.col = 0;
                }
            }
        } else if path.is_file() {
            let filename_str = path.to_string_lossy().to_string();
            if let Err(e) = self.open_buffered_file(&filename_str) {
                self.command_history.push((format!("Failed to open file: {}", e), false));
            } else {
                let full_text = self.editor.lines.join("\n");
                let _ = self.lsp_client.open_file(&filename_str, &full_text);
            }
        }
    }
}

struct Layout {
    term_width: u16,
    term_height: u16,
    banner_height: usize,
    term_pane_height: usize,
    editor_height: usize,
    debug_width: usize,
    codespace_width: usize,
    edit_pane_w: usize,
}

impl Layout {
    fn compute(state: &AppState) -> Result<Self> {
        let (term_width, term_height) = crossterm::terminal::size()?;
        let banner_height = 1;
        let status_height = 1;
        let palette_height = 1;
        let term_pane_height = if state.palette.is_active || state.show_ai_chat {
            ((term_height as usize) * 35) / 100
        } else {
            0
        };
        let editor_height = (term_height as usize).saturating_sub(term_pane_height).saturating_sub(banner_height + status_height + palette_height);
        let debug_width = if state.show_lsp_pane {
            ((term_width as usize) * 25) / 100
        } else {
            0
        };
        let codespace_width = (term_width as usize).saturating_sub(debug_width);
        let edit_pane_w = codespace_width.saturating_sub(9);
        Ok(Self {
            term_width,
            term_height,
            banner_height,
            term_pane_height,
            editor_height,
            debug_width,
            codespace_width,
            edit_pane_w,
        })
    }
}

fn set_border_color(stdout: &mut Stdout) -> Result<()> {
    stdout.execute(SetForegroundColor(colors::Palette::UI_BORDER))?;
    Ok(())
}

fn draw_frame(stdout: &mut Stdout, state: &AppState, layout: &Layout) -> Result<()> {
    set_border_color(stdout)?;
    stdout.execute(cursor::MoveTo(0, 0))?;
    write!(stdout, "╭{}╮", "─".repeat(layout.term_width.saturating_sub(2) as usize))?;
    let divider_x = layout.codespace_width.saturating_sub(1) as u16;
    for i in 0..layout.editor_height {
        let y = (layout.banner_height + i) as u16;
        stdout.execute(cursor::MoveTo(0, y))?;
        write!(stdout, "│")?;
        if state.show_lsp_pane && layout.debug_width > 0 {
            stdout.execute(cursor::MoveTo(divider_x, y))?;
            write!(stdout, "│")?;
        }
        stdout.execute(cursor::MoveTo(layout.term_width - 1, y))?;
        write!(stdout, "│")?;
    }
    let status_y = (layout.banner_height + layout.editor_height) as u16;
    stdout.execute(cursor::MoveTo(0, status_y))?;
    write!(stdout, "├{}┤", "─".repeat(layout.term_width.saturating_sub(2) as usize))?;
    for i in 0..layout.term_pane_height {
        let y = status_y + 1 + i as u16;
        stdout.execute(cursor::MoveTo(0, y))?;
        write!(stdout, "│")?;
        stdout.execute(cursor::MoveTo(layout.term_width - 1, y))?;
        write!(stdout, "│")?;
    }
    let bottom_y = layout.term_height.saturating_sub(1);
    stdout.execute(cursor::MoveTo(0, bottom_y))?;
    write!(stdout, "╰{}╯", "─".repeat(layout.term_width.saturating_sub(2) as usize))?;
    if state.show_lsp_pane && layout.debug_width > 0 {
        stdout.execute(cursor::MoveTo(divider_x, 0))?;
        write!(stdout, "┬")?;
        stdout.execute(cursor::MoveTo(divider_x, status_y))?;
        write!(stdout, "┴")?;
    }
    stdout.execute(ResetColor)?;
    Ok(())
}

fn render_ui(stdout: &mut Stdout, state: &mut AppState, layout: &Layout) -> Result<()> {
    stdout.execute(cursor::Hide)?;
    stdout.execute(SetBackgroundColor(colors::Palette::BG_MAIN))?;
    stdout.execute(Clear(ClearType::All))?;
    draw_frame(stdout, state, layout)?;
    draw_banner(stdout, layout)?;
    draw_editor(stdout, state, layout)?;
    draw_lsp_pane(stdout, state, layout)?;
    draw_status_bar(stdout, state, layout)?;
    if state.palette.is_active && layout.term_pane_height > 0 {
        draw_terminal_pane(stdout, state, layout)?;
    } else if state.show_ai_chat && layout.term_pane_height > 0 {
        draw_ai_chat_pane(stdout, state, layout)?;
    }
    draw_command_palette(stdout, state, layout)?;
    draw_symbol_finder_modal(stdout, state, layout)?;
    sync_cursor(stdout, state, layout)?;
    stdout.execute(cursor::Show)?;
    stdout.flush()?;
    Ok(())
}

fn draw_banner(stdout: &mut Stdout, _layout: &Layout) -> Result<()> {
    stdout.execute(cursor::MoveTo(2, 0))?;
    stdout.execute(SetForegroundColor(colors::Palette::FUNCTION_BLUE))?;
    write!(stdout, " Your shortcut to an Easier Life")?;
    stdout.execute(ResetColor)?;
    Ok(())
}

fn draw_editor(stdout: &mut Stdout, state: &AppState, layout: &Layout) -> Result<()> {
    for i in 0..layout.editor_height {
        let row = state.editor.row_offset + i;
        let screen_y = (layout.banner_height + i) as u16;
        stdout.execute(cursor::MoveTo(2, screen_y))?;
        if row < state.editor.lines.len() {
            let line = &state.editor.lines[row];
            let display_line: String = line.chars().skip(state.editor.col_offset).take(layout.edit_pane_w).collect();
            stdout.execute(SetForegroundColor(colors::Palette::COMMENT_GRAY))?;
            write!(stdout, "{:3}", row + 1)?;
            set_border_color(stdout)?;
            write!(stdout, "│ ")?;
            let trimmed = line.trim();
            let is_function_header = trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") || trimmed.starts_with("async fn ") || trimmed.starts_with("pub async fn ");
            if is_function_header {
                stdout.execute(SetForegroundColor(colors::Palette::SOFT_NAVY))?;
                write!(stdout, "{}", display_line)?;
                stdout.execute(ResetColor)?;
            } else {
                let mut in_string = false;
                let mut in_comment = false;
                let mut word = String::new();
                let chars: Vec<char> = display_line.chars().collect();
                for char_idx in 0..chars.len() {
                    let c = chars[char_idx];
                    let next_c = chars.get(char_idx + 1).copied();
                    if in_comment {
                        stdout.execute(SetForegroundColor(colors::Palette::COMMENT_GRAY))?;
                        write!(stdout, "{}", c)?;
                    } else if in_string {
                        stdout.execute(SetForegroundColor(colors::Palette::STRING_GREEN))?;
                        write!(stdout, "{}", c)?;
                        if c == '"' { in_string = false; }
                    } else if c == '/' && next_c == Some('/') {
                        let _ = flush_token(stdout, &mut word, Some('/'));
                        in_comment = true;
                        stdout.execute(SetForegroundColor(colors::Palette::COMMENT_GRAY))?;
                        write!(stdout, "{}", c)?;
                    } else if c == '"' {
                        let _ = flush_token(stdout, &mut word, Some('"'));
                        in_string = true;
                        stdout.execute(SetForegroundColor(colors::Palette::STRING_GREEN))?;
                        write!(stdout, "{}", c)?;
                    } else if c.is_alphanumeric() || c == '_' {
                        word.push(c);
                    } else {
                        let _ = flush_token(stdout, &mut word, Some(c));
                        stdout.execute(SetForegroundColor(colors::Palette::TEXT_DEFAULT))?;
                        write!(stdout, "{}", c)?;
                    }
                }
                let _ = flush_token(stdout, &mut word, None);
                stdout.execute(ResetColor)?;
            }
        } else {
            stdout.execute(SetForegroundColor(colors::Palette::COMMENT_GRAY))?;
            write!(stdout, "")?;
            set_border_color(stdout)?;
            write!(stdout, "│ ")?;
            stdout.execute(SetForegroundColor(colors::Palette::COMMENT_GRAY))?;
            write!(stdout, "~")?;
            stdout.execute(ResetColor)?;
        }
    }
    Ok(())
}

fn draw_lsp_pane(stdout: &mut Stdout, state: &mut AppState, layout: &Layout) -> Result<()> {
    if !state.show_lsp_pane || layout.debug_width == 0 { return Ok(()); }
    let divider_x = layout.codespace_width.saturating_sub(1) as u16;
    let debug_x = divider_x + 1;
    for i in 0..layout.editor_height {
        let y = (layout.banner_height + i) as u16;
        stdout.execute(cursor::MoveTo(debug_x, y))?;
        stdout.execute(SetBackgroundColor(colors::Palette::BG_PANEL))?;
        write!(stdout, "{}", "".repeat(layout.debug_width - 1))?;
        stdout.execute(SetBackgroundColor(colors::Palette::BG_MAIN))?;
    }
    stdout.execute(cursor::MoveTo(debug_x + 1, 1))?;
    stdout.execute(SetBackgroundColor(colors::Palette::BG_PANEL))?;
    stdout.execute(SetForegroundColor(colors::Palette::HINT_TEAL))?;
    write!(stdout, "LSP Workspace")?;
    stdout.execute(ResetColor)?;
    if state.current_diagnostics.is_empty() {
        stdout.execute(cursor::MoveTo(debug_x + 1, 3))?;
        stdout.execute(SetBackgroundColor(colors::Palette::BG_PANEL))?;
        stdout.execute(SetForegroundColor(colors::Palette::SUCCESS_GREEN))?;
        write!(stdout, "Nominal")?;
        stdout.execute(ResetColor)?;
    } else {
        let mut wrapped_lines = Vec::new();
        let text_width = layout.debug_width.saturating_sub(4);
        if text_width > 0 {
            let mut sorted_uris: Vec<_> = state.current_diagnostics.keys().collect();
            sorted_uris.sort();
            for uri in sorted_uris {
                let diags = &state.current_diagnostics[uri];
                let filename = uri.rsplit('/').next().unwrap_or("?");
                for diag in diags {
                    let status = match diag.severity {
                        Some(1) => colors::Status::Error,
                        Some(2) => colors::Status::Warning,
                        Some(3) | Some(4) => colors::Status::Hint,
                        _ => {
                            let msg_lower = diag.message.to_lowercase();
                            if msg_lower.contains("error") { colors::Status::Error }
                            else if msg_lower.contains("warn") { colors::Status::Warning }
                            else if msg_lower.contains("hint") || msg_lower.contains("info") { colors::Status::Hint }
                            else { colors::Status::Error }
                        }
                    };
                    let full_msg = format!("{}:L{}: {}", filename, diag.line, diag.message);
                    for chunk in full_msg.chars().collect::<Vec<_>>().chunks(text_width) {
                        wrapped_lines.push((chunk.iter().collect::<String>(), status));
                    }
                    wrapped_lines.push((String::new(), status));
                }
            }
        }
        let max_display_lines = layout.editor_height.saturating_sub(3);
        let max_scroll = wrapped_lines.len().saturating_sub(max_display_lines);
        state.lsp_scroll_offset = state.lsp_scroll_offset.min(max_scroll);
        for (idx, (line, status)) in wrapped_lines.iter().skip(state.lsp_scroll_offset).take(max_display_lines).enumerate() {
            stdout.execute(cursor::MoveTo(debug_x + 1, 3 + idx as u16))?;
            stdout.execute(SetBackgroundColor(colors::Palette::BG_PANEL))?;
            if !line.is_empty() {
                stdout.execute(SetAttribute(colors::attribute_for_status(*status)))?;
                stdout.execute(SetForegroundColor(colors::color_for_status(*status)))?;
                write!(stdout, "{}", line)?;
                stdout.execute(SetAttribute(Attribute::Reset))?;
                stdout.execute(ResetColor)?;
            }
        }
    }
    Ok(())
}

fn draw_status_bar(stdout: &mut Stdout, state: &AppState, layout: &Layout) -> Result<()> {
    let status_row = (layout.banner_height + layout.editor_height) as u16;
    stdout.execute(cursor::MoveTo(2, status_row))?;
    let dirty_flag = if state.editor.is_dirty { " [+] " } else { "" };
    let status = format!(
        "{}{} | R:{} C:{} | {} ({}) ",
        state.editor.filename.as_deref().unwrap_or("[Workspace]"),
        dirty_flag,
        state.editor.cursor.row + 1,
        state.editor.cursor.col + 1,
        state.lsp_client.server_name.as_str(),
        state.lsp_client.language_id.as_str(),
    );
    stdout.execute(SetForegroundColor(colors::Palette::TEXT_DEFAULT))?;
    write!(stdout, "{}", status)?;
    stdout.execute(ResetColor)?;
    Ok(())
}

fn draw_terminal_pane(stdout: &mut Stdout, state: &AppState, layout: &Layout) -> Result<()> {
    let status_row = (layout.banner_height + layout.editor_height) as u16;
    let term_start_row = status_row + 1;
    for i in 0..layout.term_pane_height {
        let y = term_start_row + i as u16;
        stdout.execute(cursor::MoveTo(1, y))?;
        stdout.execute(SetBackgroundColor(colors::Palette::BG_PANEL))?;
        write!(stdout, "{}", "".repeat(layout.term_width.saturating_sub(2) as usize))?;
    }
    let mut display_history = Vec::new();
    for (msg, success) in &state.command_history {
        for line in msg.lines() {
            display_history.push((line.to_string(), *success));
        }
    }
    let term_output_height = layout.term_pane_height;
    let max_scroll_y = display_history.len().saturating_sub(term_output_height);
    let current_scroll_y = state.term_scroll_y.min(max_scroll_y);
    let start_idx = display_history.len().saturating_sub(term_output_height).saturating_sub(current_scroll_y);
    let max_width = layout.term_width.saturating_sub(4) as usize;
    for (idx, (line, success)) in display_history.iter().skip(start_idx).take(term_output_height).enumerate() {
        stdout.execute(cursor::MoveTo(2, term_start_row + idx as u16))?;
        let status_type = if *success {
            colors::Status::Success
        } else if line.starts_with("Unknown") {
            colors::Status::UnknownCommand
        } else if line.starts_with("Code") {
            colors::Status::UnknownCode
        } else if line.contains("warn") || line.contains("Warning") {
            colors::Status::Warning
        } else if line.contains("hint") || line.contains("Hint") {
            colors::Status::Hint
        } else {
            colors::Status::Error
        };
        let chars: Vec<char> = line.chars().collect();
        let display_str: String = if state.term_scroll_x < chars.len() {
            chars.into_iter().skip(state.term_scroll_x).take(max_width).collect()
        } else {
            String::new()
        };
        stdout.execute(SetBackgroundColor(colors::Palette::BG_PANEL))?;
        stdout.execute(SetAttribute(colors::attribute_for_status(status_type)))?;
        stdout.execute(SetForegroundColor(colors::color_for_status(status_type)))?;
        write!(stdout, "{}", display_str)?;
        stdout.execute(SetAttribute(Attribute::Reset))?;
        stdout.execute(ResetColor)?;
    }
    Ok(())
}

fn draw_ai_chat_pane(stdout: &mut Stdout, state: &AppState, layout: &Layout) -> Result<()> {
    let status_row = (layout.banner_height + layout.editor_height) as u16;
    let term_start_row = status_row + 1;

    for i in 0..layout.term_pane_height {
        let y = term_start_row + i as u16;
        stdout.execute(cursor::MoveTo(1, y))?;
        stdout.execute(SetBackgroundColor(colors::Palette::BG_PANEL))?;
        write!(stdout, "{}", " ".repeat(layout.term_width.saturating_sub(2) as usize))?;
    }

    stdout.execute(cursor::MoveTo(2, term_start_row))?;
    stdout.execute(SetBackgroundColor(colors::Palette::BG_PANEL))?;
    stdout.execute(SetForegroundColor(colors::Palette::FUNCTION_BLUE))?;
    write!(stdout, "💬 ELIDE AI Auditor Chat (Ctrl+A to close)")?;
    stdout.execute(ResetColor)?;

    let max_display_lines = layout.term_pane_height.saturating_sub(2);
    let mut display_lines = Vec::new();

    for (role, msg) in &state.ai_messages {
        let prefix = if role == "user" { "You: " } else { "ELIDE: " };
        for line in msg.lines() {
            display_lines.push(format!("{}{}", prefix, line));
        }
    }

    display_lines.push(format!("> {}", state.ai_input_buffer));

    let max_scroll = display_lines.len().saturating_sub(max_display_lines);
    let scroll = state.ai_scroll_offset.min(max_scroll);
    let max_width = layout.term_width.saturating_sub(4) as usize;

    for (idx, line) in display_lines.iter().skip(scroll).take(max_display_lines).enumerate() {
        stdout.execute(cursor::MoveTo(2, term_start_row + 1 + idx as u16))?;
        stdout.execute(SetBackgroundColor(colors::Palette::BG_PANEL))?;
        
        let chars: Vec<char> = line.chars().collect();
        let display_str: String = if chars.len() > max_width {
            chars.into_iter().take(max_width).collect()
        } else {
            line.clone()
        };

        if line.starts_with("ELIDE:") {
            stdout.execute(SetForegroundColor(colors::Palette::STRING_GREEN))?;
        } else if line.starts_with("You:") {
            stdout.execute(SetForegroundColor(colors::Palette::TYPE_YELLOW))?;
        } else {
            stdout.execute(SetForegroundColor(colors::Palette::TEXT_DEFAULT))?;
        }

        write!(stdout, "{}", display_str)?;
        stdout.execute(ResetColor)?;
    }

    Ok(())
}

fn draw_command_palette(stdout: &mut Stdout, state: &AppState, layout: &Layout) -> Result<()> {
    let bottom_y = layout.term_height.saturating_sub(1);
    stdout.execute(cursor::MoveTo(2, bottom_y))?;
    if state.palette.is_active {
        stdout.execute(SetForegroundColor(colors::Palette::TYPE_YELLOW))?;
        write!(stdout, " : {}", state.palette.input_buffer)?;
        stdout.execute(ResetColor)?;
    } else {
        stdout.execute(SetForegroundColor(colors::Palette::COMMENT_GRAY))?;
        write!(stdout, " : (Ctrl+A AI chat, Alt+T term, Alt+D symbols, Esc close) ")?;
        stdout.execute(ResetColor)?;
    }
    Ok(())
}

fn draw_symbol_finder_modal(stdout: &mut Stdout, state: &AppState, layout: &Layout) -> Result<()> {
    if !state.show_symbol_finder { return Ok(()); }
    let modal_w = (layout.term_width as usize * 60) / 100;
    let modal_h = 12;
    let start_x = (layout.term_width as usize).saturating_sub(modal_w) / 2;
    let start_y = (layout.term_height as usize).saturating_sub(modal_h) / 2;
    set_border_color(stdout)?;
    stdout.execute(cursor::MoveTo(start_x as u16, start_y as u16))?;
    write!(stdout, "╭{}╮", "─".repeat(modal_w.saturating_sub(2)))?;
    for i in 1..modal_h - 1 {
        stdout.execute(cursor::MoveTo(start_x as u16, (start_y + i) as u16))?;
        stdout.execute(SetBackgroundColor(colors::Palette::BG_PANEL))?;
        write!(stdout, "│{}│", "".repeat(modal_w.saturating_sub(2)))?;
    }
    stdout.execute(cursor::MoveTo(start_x as u16, (start_y + modal_h - 1) as u16))?;
    stdout.execute(SetBackgroundColor(colors::Palette::BG_MAIN))?;
    write!(stdout, "╰{}╯", "─".repeat(modal_w.saturating_sub(2)))?;
    stdout.execute(cursor::MoveTo((start_x + 2) as u16, (start_y + 1) as u16))?;
    stdout.execute(SetBackgroundColor(colors::Palette::BG_PANEL))?;
    stdout.execute(SetForegroundColor(colors::Palette::TYPE_YELLOW))?;
    write!(stdout, "🔍 Function: {}", state.symbol_query)?;
    stdout.execute(cursor::MoveTo((start_x + 1) as u16, (start_y + 2) as u16))?;
    set_border_color(stdout)?;
    write!(stdout, "{}", "─".repeat(modal_w.saturating_sub(2)))?;
    let matches = state.get_filtered_symbols();
    let display_limit = modal_h.saturating_sub(4);
    for (idx, (_, label)) in matches.iter().take(display_limit).enumerate() {
        stdout.execute(cursor::MoveTo((start_x + 2) as u16, (start_y + 3 + idx) as u16))?;
        stdout.execute(SetBackgroundColor(colors::Palette::BG_PANEL))?;
        if idx == state.symbol_selected_idx {
            stdout.execute(SetForegroundColor(colors::Palette::FUNCTION_BLUE))?;
            write!(stdout, "► {:<width$}", label, width = modal_w.saturating_sub(5))?;
        } else {
            stdout.execute(SetForegroundColor(colors::Palette::TEXT_DEFAULT))?;
            write!(stdout, "{:<width$}", label, width = modal_w.saturating_sub(5))?;
        }
    }
    stdout.execute(ResetColor)?;
    Ok(())
}

fn sync_cursor(stdout: &mut Stdout, state: &AppState, layout: &Layout) -> Result<()> {
    if state.palette.is_active {
        let prompt_len = 3;
        stdout.execute(cursor::MoveTo((2 + prompt_len + state.palette.input_buffer.len()) as u16, layout.term_height.saturating_sub(1)))?;
    } else {
        let screen_row = (state.editor.cursor.row.saturating_sub(state.editor.row_offset) + layout.banner_height) as u16;
        let visual_col = state.editor.visual_cursor_col().saturating_sub(state.editor.col_offset);
        let screen_col = (visual_col + 8).min(layout.codespace_width.saturating_sub(2)) as u16;
        if (screen_row as usize) < (layout.banner_height + layout.editor_height) {
            stdout.execute(cursor::MoveTo(screen_col, screen_row))?;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let _guard = terminal::TerminalGuard::init()?;
    let tracer = tracer::KernelTracer::init();
    tracer.log_event("ELIDE v2.3.0 Initialized!");

    let mut stdout = stdout();
    let mut state = AppState::new();
    let (diag_tx, mut diag_rx) = mpsc::unbounded_channel::<(String, Vec<diagnostics::Diagnostic>)>();

    if let Some(arg) = std::env::args().nth(1) {
        let path = Path::new(&arg);
        if path.is_dir() {
            let canonical_dir = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
            let _ = std::env::set_current_dir(&canonical_dir);
            if let Ok(entries) = std::fs::read_dir(&canonical_dir) {
                let mut lines = vec![
                    format!(" Workspace: {}", canonical_dir.display()),
                    " ──────────────────────────────────────────".to_string(),
                    " ../".to_string(),
                ];
                let mut sorted_entries: Vec<String> = entries.flatten().filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    if name.starts_with('.') || name == "target" || name == "node_modules" { return None; }
                    if e.path().is_dir() { Some(format!("{}/", name)) } else { Some(name.to_string()) }
                }).collect();
                sorted_entries.sort();
                lines.extend(sorted_entries);
                state.editor.lines = lines;
                state.editor.filename = None;
            }
        } else if path.is_file() {
            if let Ok(content) = std::fs::read_to_string(path) {
                state.editor.lines = content.lines().map(String::from).collect();
                if state.editor.lines.is_empty() {
                    state.editor.lines.push(String::new());
                }
                let canonical_arg = AppState::normalize_path(&arg);
                state.editor.filename = Some(canonical_arg.clone());
                state.buffer_cache.insert(canonical_arg, state.editor.lines.clone());
            }
        }
    }

    let mut needs_redraw = true;
    loop {
        if state.should_exit { break; }

        while let Ok((uri, diags)) = diag_rx.try_recv() {
            if diags.is_empty() {
                state.current_diagnostics.remove(&uri);
            } else {
                state.current_diagnostics.insert(uri, diags);
            }
            needs_redraw = true;
        }

        if needs_redraw {
            let layout = Layout::compute(&state)?;
            state.editor.scroll_into_view(layout.edit_pane_w, layout.editor_height);
            render_ui(&mut stdout, &mut state, &layout)?;
            needs_redraw = false;
        }

        match terminal::poll_event(Duration::from_millis(16))? {
            InputEvent::Key(key) => {
                needs_redraw = true;
                if !state.handle_input(key, &diag_tx).await {
                    break;
                }
            }
            InputEvent::Paste(text) => {
                needs_redraw = true;
                if state.palette.is_active {
                    state.palette.input_buffer.push_str(&text);
                } else if state.show_ai_chat {
                    state.ai_input_buffer.push_str(&text);
                } else if state.editor.filename.is_some() {
                    for c in text.chars() {
                        if c == '\n' {
                            state.editor.insert_newline();
                        } else if c != '\r' {
                            state.editor.insert_char(c);
                        }
                    }
                }
            }
            InputEvent::ScrollUp => {
                needs_redraw = true;
                if state.palette.is_active {
                    state.term_scroll_y = state.term_scroll_y.saturating_add(3);
                } else if state.show_ai_chat {
                    state.ai_scroll_offset = state.ai_scroll_offset.saturating_add(3);
                } else if state.show_lsp_pane {
                    state.lsp_scroll_offset = state.lsp_scroll_offset.saturating_sub(3);
                } else {
                    state.editor.move_cursor(-3, 0);
                }
            }
            InputEvent::ScrollDown => {
                needs_redraw = true;
                if state.palette.is_active {
                    state.term_scroll_y = state.term_scroll_y.saturating_sub(3);
                } else if state.show_ai_chat {
                    state.ai_scroll_offset = state.ai_scroll_offset.saturating_sub(3);
                } else if state.show_lsp_pane {
                    state.lsp_scroll_offset = state.lsp_scroll_offset.saturating_add(3);
                } else {
                    state.editor.move_cursor(3, 0);
                }
            }
            InputEvent::Resize => {
                needs_redraw = true;
            }
            InputEvent::Tick => {}
        }
    }
    Ok(())
}